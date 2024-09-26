use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::punctuated::Punctuated;
use syn::{parse_macro_input, Ident, ItemFn, Token, Type, Visibility};

fn make_system_struct(func: &ItemFn) -> (Ident, TokenStream) {
    let input_name = &func.sig.ident;
    let input_vis = &func.vis;

    let tokens = quote! {
        #[allow(non_camel_case_types)]
        #input_vis struct #input_name {}
    };

    (input_name.clone(), tokens)
}

fn make_inner_system_func(func: &ItemFn) -> (Ident, TokenStream) {
    let mut new_inner_func = func.clone();
    let ident = format_ident!("__wrapped_inner_func");
    new_inner_func.sig.ident = ident.clone();
    new_inner_func.vis = Visibility::Inherited;

    (ident, new_inner_func.to_token_stream())
}

fn gather_args(func: &ItemFn) -> Vec<Type> {
    let mut ret = Vec::new();

    for arg in &func.sig.inputs {
        match arg {
            syn::FnArg::Receiver(_) => panic!("Cannot handle 'self' type in system"),
            syn::FnArg::Typed(pat_type) => {
                ret.push(*pat_type.ty.clone());
            }
        }
    }
    ret
}

fn make_outer_system_func(
    input: &ItemFn,
    system_args: &Punctuated<Type, Token![,]>,
) -> (Ident, TokenStream) {
    let (inner_system_func_ident, inner_system_func_definition) = make_inner_system_func(input);

    let arg_forwards: Punctuated<TokenStream, Token![,]> = if system_args.len() == 1 {
        vec![quote! {__system_args}].into_iter().collect()
    } else {
        (0..system_args.len())
            .map(|idx| {
                let tuple_index = syn::Index::from(idx);
                quote! {__system_args.#tuple_index}
            })
            .collect()
    };

    let outer_system_func_ident = format_ident!("{}_system_impl", input.sig.ident);

    let tokens = quote! {
        fn #outer_system_func_ident(
            __world: &::wutengine::ecs::world::World,
        ) -> ::wutengine::command::Command {
            let __commands = unsafe {
                __world.query(|__entity_id: ::wutengine::core::EntityId, __system_args: (#system_args)| {
                    let mut __command_accumulator = ::wutengine::command::Command::NONE;

                    #inner_system_func_definition

                    #inner_system_func_ident(&mut __command_accumulator, __entity_id, #arg_forwards);

                    __command_accumulator
                })
            };

            __commands
                .into_iter()
                .fold(::wutengine::command::Command::NONE, |mut a, b| {
                    a.merge_with(b);
                    a
                })
        }
    };

    (outer_system_func_ident, tokens)
}

fn make_description_impl(
    struct_ident: Ident,
    func_ident: Ident,
    component_types: &Punctuated<Type, Token![,]>,
) -> TokenStream {
    quote! {
        impl ::wutengine::ecs::FunctionDescription for #struct_ident {
            fn describe() -> ::wutengine::ecs::SystemFunctionDescriptor {
                ::wutengine::ecs::SystemFunctionDescriptor {
                    read_writes: <(#component_types) as ::wutengine::ecs::world::CombinedQuery>::get_descriptors(),
                    func: #func_ident,
                }
            }
        }
    }
}

///TODO: Give better error if first two arguments are not command and entity
#[proc_macro_attribute]
pub fn system(
    _args: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    let system_args: Punctuated<Type, Token![,]> =
        gather_args(&input).into_iter().skip(2).collect();
    let (system_struct_name, system_struct_def) = make_system_struct(&input);
    let (outer_system_func_ident, outer_system_func_def) =
        make_outer_system_func(&input, &system_args);
    let description_impl =
        make_description_impl(system_struct_name, outer_system_func_ident, &system_args);

    quote! {
        #system_struct_def

        #outer_system_func_def

        #description_impl
    }
    .into()
}
