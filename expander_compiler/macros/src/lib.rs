use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, FnArg, ItemFn, PatType, Type};

fn calculate_array_total_len(ty: &Type) -> proc_macro2::TokenStream {
    match ty {
        Type::Array(array) => {
            let len = &array.len;
            let inner_len = calculate_array_total_len(&array.elem);
            quote! { (#len as usize) * (#inner_len) }
        }
        _ => quote! { 1usize },
    }
}

fn replace_array_with_vec(ty: &Type) -> proc_macro2::TokenStream {
    match ty {
        Type::Array(array) => {
            let elem = replace_array_with_vec(&array.elem);
            quote! { Vec<#elem> }
        }
        Type::Path(path) => {
            let ident = &path.path.segments.last().unwrap().ident;
            match ident.to_string().as_str() {
                "InputVariable" | "OutputVariable" | "InputOutputVariable" => {
                    quote! { Variable }
                }
                _ => panic!("Unsupported type: {}. Expected InputVariable, OutputVariable, or InputOutputVariable", ident),
            }
        }
        _ => panic!("Unsupported type structure. Expected array of Variable types."),
    }
}

fn get_variable_spec(ty: &Type) -> (bool, bool) {
    if let Type::Path(path) = ty {
        let ident = &path.path.segments.last().unwrap().ident;
        match ident.to_string().as_str() {
            "InputVariable" => (true, false),
            "OutputVariable" => (false, true),
            "InputOutputVariable" => (true, true),
            _ => panic!("Unsupported variable type. Expected InputVariable, OutputVariable, or InputOutputVariable"),
        }
    } else {
        panic!("Expected a Variable type")
    }
}

fn generate_unflatten_code(
    name: &proc_macro2::TokenStream,
    ty: &Type,
    offset: &proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    match ty {
        Type::Array(array) => {
            let len = &array.len;
            let inner_code = generate_unflatten_code(name, &array.elem, offset);
            quote! {
                {
                    let mut temp = Vec::with_capacity(#len as usize);
                    let mut local_offset = #offset;
                    for _ in 0..#len {
                        temp.push(#inner_code);
                        local_offset += calculate_inner_len(&#array.elem);
                    }
                    temp
                }
            }
        }
        Type::Path(_) => {
            quote! { #name[#offset].clone() }
        }
        _ => panic!("Unexpected type in unflatten"),
    }
}

fn generate_flatten_code(name: &proc_macro2::TokenStream, ty: &Type) -> proc_macro2::TokenStream {
    match ty {
        Type::Array(array) => {
            let inner_code = generate_flatten_code(&quote! { inner }, &array.elem);
            quote! {
                {
                    let mut flat = Vec::new();
                    for inner in #name.iter() {
                        flat.extend(#inner_code);
                    }
                    flat
                }
            }
        }
        Type::Path(_) => {
            quote! { vec![#name.clone()] }
        }
        _ => panic!("Unexpected type in flatten"),
    }
}

#[proc_macro_attribute]
pub fn kernel(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &input_fn.sig.ident;
    let user_fn_name = format_ident!("user_{}", fn_name);
    let compile_fn_name = format_ident!("compile_{}", fn_name);
    let stmts = &input_fn.block.stmts;

    let mut specs = Vec::new();
    let mut arg_names = Vec::new();
    let mut unflatten_code = Vec::new();
    let mut flatten_code = Vec::new();
    let mut current_index = 0;

    let api_arg = input_fn
        .sig
        .inputs
        .first()
        .expect("Expected at least one argument (API)");

    let user_fn_inputs: Vec<_> = input_fn
        .sig
        .inputs
        .iter()
        .skip(1)
        .map(|arg| {
            if let FnArg::Typed(PatType { pat, ty, .. }) = arg {
                if let Type::Reference(ref_type) = &**ty {
                    if let Type::Array(array) = &*ref_type.elem {
                        let mut inner_ty = &*array.elem;
                        while let Type::Array(arr) = inner_ty {
                            inner_ty = &*arr.elem;
                        }

                        let (is_input, is_output) = get_variable_spec(inner_ty);
                        let total_len = calculate_array_total_len(&ref_type.elem);

                        specs.push(quote! {
                            IOVecSpec {
                                len: #total_len,
                                is_input: #is_input,
                                is_output: #is_output,
                            }
                        });

                        let arg_name = quote! { #pat };
                        arg_names.push(arg_name.clone());

                        let unflatten = generate_unflatten_code(
                            &quote! { inputs[#current_index] },
                            &ref_type.elem,
                            &quote! { 0 },
                        );
                        unflatten_code.push(quote! {
                            let #pat = #unflatten;
                        });

                        if is_output {
                            let flatten = generate_flatten_code(&arg_name, &ref_type.elem);
                            flatten_code.push(quote! {
                                inputs[#current_index] = #flatten;
                            });
                        }

                        current_index += 1;

                        let vec_type = replace_array_with_vec(&ref_type.elem);
                        if ref_type.mutability.is_some() {
                            quote! { #pat: &mut #vec_type }
                        } else {
                            quote! { #pat: &#vec_type }
                        }
                    } else {
                        panic!("Expected an array type for kernel parameters");
                    }
                } else {
                    panic!("Expected a reference type for kernel parameters");
                }
            } else {
                panic!("Unsupported argument type for kernel function");
            }
        })
        .collect();

    let expanded = quote! {
        fn #user_fn_name<C: Config>(#api_arg, #(#user_fn_inputs),*) {
            #(#stmts)*
        }

        fn #compile_fn_name<C: Config>() -> Kernel<C> {
            compile_with_spec(
                |api: &mut API<C>, inputs: &mut Vec<Vec<Variable>>| {
                    #(#unflatten_code)*

                    #user_fn_name(api, #(&#arg_names),*);

                    #(#flatten_code)*
                },
                &[#(#specs),*]
            )
            .unwrap()
        }
    };

    TokenStream::from(expanded)
}
