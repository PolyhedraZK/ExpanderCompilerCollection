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

    let user_fn_inputs = input_fn.sig.inputs.iter().skip(1).collect::<Vec<_>>();
    let return_type = &input_fn.sig.output;

    let mut param_infos = Vec::new();
    for arg in input_fn.sig.inputs.iter().skip(1) {
        if let FnArg::Typed(PatType { pat, ty, .. }) = arg {
            let param_name = quote!(#pat).to_string();

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

    let expanded = quote! {
        fn #user_fn_name #generics (#api_arg, #(#user_fn_inputs),*) #return_type {
            #(#stmts)*
        }
    };

    eprintln!("Expanded code: {}", expanded);
    TokenStream::from(expanded)
}
