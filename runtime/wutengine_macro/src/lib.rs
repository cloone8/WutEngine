#![doc = include_str!("../README.md")]

use quote::quote;
use syn::ItemFn;
use syn::LitStr;
use syn::meta;
use syn::parse_macro_input;

/// Macro to attach to the main game entrypoint. Automatically emits a proper `main` function for the current target platform
#[proc_macro_attribute]
pub fn main(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut org_name: Option<LitStr> = None;
    let mut app_name: Option<LitStr> = None;

    let arg_parser = meta::parser(|meta| {
        if meta.path.is_ident("org") {
            match org_name {
                Some(_) => {
                    return Err(meta.error("org already set"));
                }
                None => {
                    org_name = Some(meta.value()?.parse::<LitStr>()?);
                }
            }
        } else if meta.path.is_ident("app") {
            match app_name {
                Some(_) => {
                    return Err(meta.error("app already set"));
                }
                None => {
                    app_name = Some(meta.value()?.parse::<LitStr>()?);
                }
            }
        } else {
            return Err(meta.error("Unknown option"));
        }

        Ok(())
    });

    parse_macro_input!(attr with arg_parser);

    let org_name = if let Some(org_name) = org_name {
        quote! {#org_name}
    } else {
        quote! { "UnknownOrg" }
    };

    let app_name = if let Some(app_name) = app_name {
        quote! {#app_name}
    } else {
        quote! { env!("CARGO_PKG_NAME") }
    };

    let app_entrypoint_item = parse_macro_input!(item as ItemFn);
    let app_entrypoint_name = app_entrypoint_item.sig.ident.clone();

    quote! {
        fn main() {
            #app_entrypoint_item

            let opts = ::wutengine::runtime::StartupOptions {
                org_name: #org_name,
                app_name: #app_name
            };

            ::wutengine::runtime::_wutengine_entry(&opts, #app_entrypoint_name);
        }


    }
    .into()
}
