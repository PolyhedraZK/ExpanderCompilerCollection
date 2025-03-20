use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, FnArg, GenericParam, Ident, ItemFn, PatType, Path, PathSegment, ReturnType,
    Type, TypeParam,
};

#[derive(Debug)]
enum ParamKind {
    Variable,
    Primitive,
}

#[derive(Debug)]
struct ParamInfo {
    name: String,
    kind: ParamKind,
    vec_depth: usize,
    is_ref: bool,
}

fn check_type_variant(ident: &Ident, allow_primitive: bool) -> Option<ParamKind> {
    if ident == "Variable" {
        Some(ParamKind::Variable)
    } else if allow_primitive
        && matches!(
            ident.to_string().as_str(),
            "u64" | "u32" | "usize" | "i64" | "i32" | "isize"
        )
    {
        Some(ParamKind::Primitive)
    } else {
        None
    }
}

fn analyze_type_structure(ty: &Type, allow_primitive: bool) -> Option<(ParamKind, usize, bool)> {
    let mut is_ref = false;
    let mut current_type = ty;

    if let Type::Reference(type_ref) = current_type {
        is_ref = true;
        current_type = type_ref.elem.as_ref();
    }

    if let Type::Path(type_path) = current_type {
        let segments = &type_path.path.segments;
        let mut vec_depth = 0;
        let mut current_segments = segments;

        if let Some(last_segment) = current_segments.last() {
            if let Some(kind) = check_type_variant(&last_segment.ident, allow_primitive) {
                return Some((kind, 0, is_ref));
            }
        }

        while let Some(last_segment) = current_segments.last() {
            if last_segment.ident == "Vec" {
                vec_depth += 1;
                if let syn::PathArguments::AngleBracketed(args) = &last_segment.arguments {
                    if let Some(syn::GenericArgument::Type(Type::Path(inner_path))) =
                        args.args.first()
                    {
                        current_segments = &inner_path.path.segments;
                        continue;
                    }
                }
                return None;
            } else {
                if let Some(kind) = check_type_variant(&last_segment.ident, allow_primitive) {
                    return Some((kind, vec_depth, is_ref));
                }
                return None;
            }
        }
    }
    None
}

#[proc_macro_attribute]
pub fn memorized(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &input_fn.sig.ident;
    let user_fn_name = format_ident!("{}", fn_name);
    let memorized_fn_name = format_ident!("memorized_{}", fn_name);
    let stmts = &input_fn.block.stmts;

    let generics = &input_fn.sig.generics;
    let api_arg = input_fn
        .sig
        .inputs
        .first()
        .expect("Expected at least one argument (API)");
    
    let api_arg_name = if let FnArg::Typed(PatType { pat, .. }) = api_arg {
        quote!(#pat).to_string().replace(" ", "")
    } else {
        panic!("API argument must be a typed parameter")
    };
    let api_arg_ident = format_ident!("{}", api_arg_name);

    let user_fn_inputs = input_fn.sig.inputs.iter().skip(1).collect::<Vec<_>>();
    let return_type = &input_fn.sig.output;

    let mut param_infos = Vec::new();
    for arg in input_fn.sig.inputs.iter().skip(1) {
        if let FnArg::Typed(PatType { pat, ty, .. }) = arg {
            let param_name = quote!(#pat).to_string().replace(" ", "");

            if let Some((kind, vec_depth, is_ref)) = analyze_type_structure(&ty, true) {
                param_infos.push(ParamInfo {
                    name: param_name,
                    kind,
                    vec_depth,
                    is_ref,
                });
            } else {
                panic!("Unsupported parameter type");
            }
        }
    }

    let return_info = match &input_fn.sig.output {
        ReturnType::Type(_, ty) => {
            if let Type::Reference(_) = **ty {
                panic!("Return type cannot be a reference");
            }

            if let Some((kind, vec_depth, is_ref)) = analyze_type_structure(ty, false) {
                Some(ParamInfo {
                    name: String::from("return"),
                    kind,
                    vec_depth,
                    is_ref,
                })
            } else {
                panic!("Return type must be Vec<...Vec<Variable>...> or Variable");
            }
        }
        ReturnType::Default => None,
    };

    eprintln!("Parameter infos: {:#?}", param_infos);
    eprintln!("Return info: {:#?}", return_info);

    let join_variable_calls = param_infos.iter()
        .filter(|param_info| matches!(param_info.kind, ParamKind::Variable))
        .map(|param_info| {
            let param_name = format_ident!("{}", param_info.name);
            quote! {
                extra::JoinVecVariables::join_vec_variables(&#param_name, &mut inputs, &mut input_structure);
            }
        });

    let hash_primitive_calls = param_infos.iter().map(|param_info| {
        let param_name = format_ident!("{}", param_info.name);
        quote! {
            extra::HashStructureAndPrimitive::hash_structure_and_primitive(&#param_name, &mut hasher);
        }
    });

    let rebuild_vars = param_infos.iter().map(|param_info| {
        let param_name = format_ident!("{}", param_info.name);
        
        match param_info.kind {
            ParamKind::Variable => {
                if param_info.vec_depth > 0 {
                    let mut type_tokens = quote! { Variable };
                    for _ in 0..param_info.vec_depth {
                        type_tokens = quote! { Vec<#type_tokens> };
                    }
                    quote! {
                        let #param_name = <#type_tokens as extra::RebuildVecVariables>::rebuild_vec_variables(&mut s, &mut structure);
                    }
                } else {
                    quote! {
                        let #param_name = <Variable as extra::RebuildVecVariables>::rebuild_vec_variables(&mut s, &mut structure);
                    }
                }
            },
            ParamKind::Primitive => {
                quote! {
                    let #param_name = #param_name;
                }
            }
        }
    });

    let user_fn_args = param_infos.iter().map(|param_info| {
        let param_name = format_ident!("{}", param_info.name);
        if param_info.is_ref {
            quote! { &#param_name }
        } else {
            quote! { #param_name }
        }
    });

    let call_user_fn = quote! {
        #user_fn_name(#api_arg_ident, #(#user_fn_args),*)
    };
    
    let join_output = if let Some(return_info) = &return_info {
        quote! {
            let result = #call_user_fn;
            let mut outputs: Vec<Variable> = Vec::new();
            let mut output_structure: Vec<usize> = Vec::new();
            extra::JoinVecVariables::join_vec_variables(&result, &mut outputs, &mut output_structure);
            #api_arg_ident.register_sub_circuit_output_structure(circuit_id, output_structure);
            outputs
        }
    } else {
        quote! {
            #call_user_fn;
            Vec::new()
        }
    };

    let rebuild_return = if let Some(return_info) = &return_info {
        let mut return_type_tokens = quote! { Variable };
        for _ in 0..return_info.vec_depth {
            return_type_tokens = quote! { Vec<#return_type_tokens> };
        }
        
        quote! {
            let mut s = outputs.as_slice();
            let structure = #api_arg_ident.get_sub_circuit_output_structure(circuit_id);
            let mut structure_slice = structure.as_slice();
            let result = <#return_type_tokens as extra::RebuildVecVariables>::rebuild_vec_variables(
                &mut s,
                &mut structure_slice,
            );
            result
        }
    } else {
        quote! { () }
    };

    let expanded = quote! {
        fn #user_fn_name #generics (#api_arg, #(#user_fn_inputs),*) #return_type {
            #(#stmts)*
        }

        fn #memorized_fn_name #generics (#api_arg, #(#user_fn_inputs),*) #return_type {
            use tiny_keccak::Hasher;
            let mut inputs: Vec<Variable> = Vec::new();
            let mut input_structure: Vec<usize> = Vec::new();
            
            #(#join_variable_calls)*
            
            let mut hasher = tiny_keccak::Keccak::v256();
            hasher.update(b"memorized");
            
            #(#hash_primitive_calls)*
            
            let mut hash = [0u8; 32];
            hasher.finalize(&mut hash);
            
            let circuit_id = #api_arg_ident.hash_to_sub_circuit_id(&hash);
            
            let f = |#api_arg, inputs: &Vec<Variable>| -> Vec<Variable> {
                let mut s = inputs.as_slice();
                let mut structure = input_structure.as_slice();
                
                #(#rebuild_vars)*
                
                #join_output
            };
            
            let outputs = #api_arg_ident.call_sub_circuit(circuit_id, &inputs, f);
            #rebuild_return
        }
    };

    eprintln!("Expanded code: {}", expanded);
    TokenStream::from(expanded)
}