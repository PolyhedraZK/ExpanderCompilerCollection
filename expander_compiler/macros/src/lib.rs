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

fn get_array_shape(ty: &Type) -> proc_macro2::TokenStream {
    match ty {
        Type::Array(array) => {
            let len = &array.len;
            let inner_shape = get_array_shape(&array.elem);
            quote! { (#len as usize) , #inner_shape }
        }
        _ => quote! {},
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

fn generate_array_access(vars: &[proc_macro2::Ident]) -> proc_macro2::TokenStream {
    let mut access = quote! {};
    for var in vars {
        access = quote! { #access[#var] };
    }
    access
}

fn generate_unflatten_code(
    array_index: usize,
    output_name: &proc_macro2::TokenStream,
    ty: &Type,
    is_input: bool,
) -> proc_macro2::TokenStream {
    fn collect_dimensions(ty: &Type) -> Vec<&syn::Expr> {
        let mut dims = Vec::new();
        let mut curr_ty = ty;
        while let Type::Array(array) = curr_ty {
            dims.push(&array.len);
            curr_ty = &array.elem;
        }
        dims
    }

    let dims = collect_dimensions(ty);
    if dims.is_empty() {
        if is_input {
            return quote! {
                let mut #output_name: Variable = inputs[#array_index][0];
            };
        }
        return quote! {
            let mut #output_name: Variable = Variable::default();
        };
    }

    let mut steps = Vec::with_capacity(dims.len());
    let mut step = quote! { 1 };
    for &dim in dims.iter().skip(1).rev() {
        step = quote! { #dim * (#step) };
        steps.push(step.clone());
    }
    steps.reverse();

    let loop_vars = (0..dims.len())
        .map(|i| format_ident!("i_{}", i))
        .collect::<Vec<_>>();

    let mut index_calc = quote! { 0 };
    for (i, (step, var)) in steps.iter().zip(loop_vars.iter()).enumerate() {
        if i == 0 {
            index_calc = quote! { #var * (#step) };
        } else {
            index_calc = quote! { #index_calc + #var * (#step) };
        }
    }
    if let Some(last_var) = loop_vars.last() {
        index_calc = quote! { #index_calc + #last_var };
    }

    let array_access = generate_array_access(&loop_vars[..loop_vars.len() - 1]);
    let mut inner_code = if is_input {
        quote! {
            #output_name #array_access.push(inputs[#array_index][#index_calc].clone());
        }
    } else {
        quote! {
            #output_name #array_access.push(Variable::default());
        }
    };

    for (i, (var, &dim)) in loop_vars.iter().zip(dims.iter()).enumerate().rev() {
        let init_code = if i == loop_vars.len() - 1 {
            quote! {}
        } else if i == 0 {
            quote! { #output_name.push(Vec::new()); }
        } else {
            let init_dims = &loop_vars[..i];
            let array_access = generate_array_access(init_dims);
            quote! { #output_name #array_access.push(Vec::new()); }
        };

        inner_code = quote! {
            for #var in 0..#dim {
                #init_code
                #inner_code
            }
        };
    }

    quote! {
        let mut #output_name: Vec<_> = Vec::new();
        #inner_code
    }
}

fn generate_flatten_code(
    array_index: usize,
    input_name: &proc_macro2::TokenStream,
    ty: &Type,
) -> proc_macro2::TokenStream {
    fn collect_dimensions(ty: &Type) -> Vec<&syn::Expr> {
        let mut dims = Vec::new();
        let mut curr_ty = ty;
        while let Type::Array(array) = curr_ty {
            dims.push(&array.len);
            curr_ty = &array.elem;
        }
        dims
    }

    let dims = collect_dimensions(ty);
    if dims.is_empty() {
        return quote! {
            inputs[#array_index][0] = #input_name;
        };
    }

    let mut steps = Vec::with_capacity(dims.len());
    let mut step = quote! { 1 };
    for &dim in dims.iter().skip(1).rev() {
        step = quote! { #dim * (#step) };
        steps.push(step.clone());
    }
    steps.reverse();

    let loop_vars = (0..dims.len())
        .map(|i| format_ident!("i_{}", i))
        .collect::<Vec<_>>();

    let mut index_calc = quote! { 0 };
    for (i, (step, var)) in steps.iter().zip(loop_vars.iter()).enumerate() {
        if i == 0 {
            index_calc = quote! { #var * (#step) };
        } else {
            index_calc = quote! { #index_calc + #var * (#step) };
        }
    }
    if let Some(last_var) = loop_vars.last() {
        index_calc = quote! { #index_calc + #last_var };
    }

    let array_access = generate_array_access(&loop_vars);
    let mut loop_code = quote! {
        inputs[#array_index][#index_calc] = #input_name #array_access.clone();
    };

    for (var, &dim) in loop_vars.iter().zip(dims.iter()).rev() {
        loop_code = quote! {
            for #var in 0..#dim {
                #loop_code
            }
        };
    }

    quote! { #loop_code }
}

#[proc_macro_attribute]
pub fn kernel(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // eprintln!("Input tokens: {:#?}", item);
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &input_fn.sig.ident;
    let user_fn_name = format_ident!("{}", fn_name);
    let compile_fn_name = format_ident!("compile_{}", fn_name);
    let stmts = &input_fn.block.stmts;

    let mut specs = Vec::new();
    let mut shapes = Vec::new();
    let mut arg_names = Vec::new();
    let mut arg_mutability = Vec::new();
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
                    let vec_type = replace_array_with_vec(&ref_type.elem);
                    let arg_name = quote! { #pat };

                    let mut inner_ty = &*ref_type.elem;
                    while let Type::Array(arr) = inner_ty {
                        inner_ty = &*arr.elem;
                    }
                    let (is_input, is_output) = get_variable_spec(inner_ty);
                    let total_len = calculate_array_total_len(&ref_type.elem);
                    let shape = get_array_shape(&ref_type.elem);
                    shapes.push(quote! { Some(vec![#shape]) });

                    specs.push(quote! {
                        IOVecSpec {
                            len: #total_len,
                            is_input: #is_input,
                            is_output: #is_output,
                        }
                    });

                    arg_names.push(arg_name.clone());

                    unflatten_code.push(generate_unflatten_code(
                        current_index,
                        &arg_name,
                        &ref_type.elem,
                        is_input,
                    ));

                    if is_output {
                        flatten_code.push(generate_flatten_code(
                            current_index,
                            &arg_name,
                            &ref_type.elem,
                        ));
                    }

                    current_index += 1;

                    arg_mutability.push(ref_type.mutability.is_some());
                    if ref_type.mutability.is_some() {
                        quote! { #pat: &mut #vec_type }
                    } else {
                        quote! { #pat: &#vec_type }
                    }
                } else {
                    panic!("Expected a reference type for kernel parameters");
                }
            } else {
                panic!("Unsupported argument type for kernel function");
            }
        })
        .collect();

    let fn_args = arg_names
        .iter()
        .zip(arg_mutability.iter())
        .map(|(name, is_mut)| {
            if *is_mut {
                quote! { &mut #name }
            } else {
                quote! { &#name }
            }
        });

    let expanded = quote! {
        fn #user_fn_name<C: Config>(#api_arg, #(#user_fn_inputs),*) {
            #(#stmts)*
        }

        fn #compile_fn_name<C: Config>() -> Result<Kernel<C>, Error> {
            compile_with_spec_and_shapes(
                |api: &mut API<C>, inputs: &mut Vec<Vec<Variable>>| {
                    #(#unflatten_code)*

                    #user_fn_name(api, #(#fn_args),*);

                    #(#flatten_code)*
                },
                &[#(#specs),*],
                &[#(#shapes),*],
            )
        }
    };

    // eprintln!("Expanded tokens: {:#?}", expanded);
    // eprintln!("Expanded code: {}", expanded);
    TokenStream::from(expanded)
}
