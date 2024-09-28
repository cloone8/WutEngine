//! Macros for users of WutEngine

#![allow(clippy::missing_docs_in_private_items)]

use darling::ast::NestedMeta;
use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::punctuated::Punctuated;
use syn::{
    parse_macro_input, Attribute, DeriveInput, Ident, ItemFn, Meta, Token, Type, Visibility,
};

fn make_system_struct(func: &ItemFn) -> (Ident, TokenStream) {
    let input_name = &func.sig.ident;
    let input_vis = &func.vis;
    let input_docs: Vec<TokenStream> = get_doc_attrs(&func.attrs)
        .into_iter()
        .map(|a| a.into_token_stream())
        .collect();
    let tokens = quote! {
        #(#input_docs)*
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

fn get_doc_attrs(attrs: &[Attribute]) -> Vec<&Attribute> {
    attrs
        .iter()
        .filter(|x| x.path().is_ident("doc"))
        .filter(|x| matches!(x.meta, Meta::NameValue(_)))
        .collect()
}

fn make_outer_system_func(
    input: &ItemFn,
    system_args: &Punctuated<Type, Token![,]>,
    args: &SystemMacroArgs,
) -> (Ident, TokenStream) {
    let root = &args.root_wutengine_crate;

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
            __world: &#root::ecs::world::World,
        ) -> #root::command::Command {
            let __commands = unsafe {
                __world.query(|__entity_id: #root::core::EntityId, __system_args: (#system_args)| {
                    let mut __command_accumulator = #root::command::Command::NONE;

                    #inner_system_func_definition

                    #inner_system_func_ident(&mut __command_accumulator, __entity_id, #arg_forwards);

                    __command_accumulator
                })
            };

            __commands
                .into_iter()
                .fold(#root::command::Command::NONE, |mut a, b| {
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
    args: &SystemMacroArgs,
) -> TokenStream {
    let root = &args.root_wutengine_crate;

    quote! {
        impl #root::ecs::FunctionDescription for #struct_ident {
            fn describe() -> #root::ecs::SystemFunctionDescriptor {
                #root::ecs::SystemFunctionDescriptor {
                    read_writes: <(#component_types) as #root::ecs::world::CombinedQuery>::get_read_write_descriptors(),
                    func: #func_ident,
                }
            }
        }
    }
}

/// Transforms a compatible function into a system with description, for use
/// in WutEngine.
///
///TODO: Give better error if first two arguments are not command and entity
#[proc_macro_attribute]
pub fn system(
    args: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    let macro_args: SystemMacroArgs = match parse_macro_args(args.into()) {
        Ok(a) => a,
        Err(e) => return e.into(),
    };

    let system_args: Punctuated<Type, Token![,]> =
        gather_args(&input).into_iter().skip(2).collect();
    let (system_struct_name, system_struct_def) = make_system_struct(&input);
    let (outer_system_func_ident, outer_system_func_def) =
        make_outer_system_func(&input, &system_args, &macro_args);
    let description_impl = make_description_impl(
        system_struct_name,
        outer_system_func_ident,
        &system_args,
        &macro_args,
    );

    quote! {
        #system_struct_def

        #outer_system_func_def

        #description_impl
    }
    .into()
}

#[derive(FromMeta, Debug)]
#[darling(default)]
struct SystemMacroArgs {
    root_wutengine_crate: syn::Path,
}

impl Default for SystemMacroArgs {
    fn default() -> Self {
        Self {
            root_wutengine_crate: syn::parse_quote!(::wutengine),
        }
    }
}

/// Attempts to parse the given proc macro args input into the given type.
/// If this can't be done (due to invalid input), a tokenstream describing
/// the error to the user is returned. This tokenstream can then be
/// emitted directly, instead of any other output.
fn parse_macro_args<T: FromMeta>(args: TokenStream) -> Result<T, TokenStream> {
    let attr_args = match NestedMeta::parse_meta_list(args) {
        Ok(args) => args,
        Err(e) => return Err(e.into_compile_error()),
    };

    let parsed = match T::from_list(&attr_args) {
        Ok(p) => p,
        Err(e) => return Err(e.write_errors()),
    };

    Ok(parsed)
}

/// Derives the `Component` trait for a type
#[proc_macro_derive(Component)]
pub fn derive_component(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let target = input.ident;

    quote! {
        impl ::wutengine::core::Component for #target {}
    }
    .into()
}
