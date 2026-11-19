#![doc = include_str!("../README.md")]

use proc_macro2::Span;
use quote::quote;
use syn::Attribute;
use syn::Ident;
use syn::ItemFn;
use syn::LitStr;
use syn::Type;
use syn::Visibility;
use syn::meta;
use syn::parse::Parse;
use syn::parse_macro_input;
use syn::parse_str;

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

/// Input for the [`unique_id_type32`] and [`unique_id_type64`] macros
struct UniqueIdTypeInput {
    /// Existing attributes to apply
    attrs: Vec<Attribute>,

    /// The visibility to assign to the generated type
    vis: Option<Visibility>,

    /// Name of the new ID type
    name: Ident,
}

impl Parse for UniqueIdTypeInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let vis: Option<Visibility> = input.parse::<Visibility>().ok();
        let name: Ident = input.parse()?;

        Ok(Self { attrs, vis, name })
    }
}

struct UniqueIdConfig {
    ident_id: Ident,
    attrs: Vec<Attribute>,
    vis: Option<Visibility>,
    atomic_type: Type,
    inner_type: Type,
    format_string: LitStr,
    hash_write: Ident,
}

/// Generates a new 32-bit atomic identifier type, which automatically increments itself
/// whenever a new instance is created.
#[proc_macro]
pub fn unique_id_type32(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as UniqueIdTypeInput);

    unique_id_type(UniqueIdConfig {
        ident_id: input.name,
        attrs: input.attrs,
        vis: input.vis,
        atomic_type: parse_str("::core::sync::atomic::AtomicU32").unwrap(),
        inner_type: parse_str("::core::num::NonZeroU32").unwrap(),
        format_string: LitStr::new("{:08x}", Span::call_site()),
        hash_write: parse_str("write_u32").unwrap(),
    })
}

/// Generates a new 64-bit atomic identifier type, which automatically increments itself
/// whenever a new instance is created.
#[proc_macro]
pub fn unique_id_type64(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as UniqueIdTypeInput);

    unique_id_type(UniqueIdConfig {
        ident_id: input.name,
        attrs: input.attrs,
        vis: input.vis,
        atomic_type: parse_str("::core::sync::atomic::AtomicU64").unwrap(),
        inner_type: parse_str("::core::num::NonZeroU64").unwrap(),
        format_string: LitStr::new("{:016x}", Span::call_site().into()),
        hash_write: parse_str("write_u64").unwrap(),
    })
}

fn unique_id_type(config: UniqueIdConfig) -> proc_macro::TokenStream {
    let ident_id = config.ident_id;

    let id_overflow_err = format!("Overflow for ID of type `{ident_id}`");

    let ident_new_doc = format!("Generate a new guaranteed unique [{ident_id}]");
    let attrs = config.attrs;
    let vis = config.vis;
    let vis_new = match &vis {
        Some(Visibility::Public(_)) => Some(parse_str("pub(crate)").unwrap()),
        Some(other) => Some(other.clone()),
        None => None,
    };
    let atomic_type = config.atomic_type;
    let inner_type = config.inner_type;
    let format_string = config.format_string;
    let hash_write = config.hash_write;

    quote! {
        #(#attrs)*
        #[repr(transparent)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
        #vis struct #ident_id(#inner_type);

        impl ::std::fmt::Display for #ident_id {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                write!(f, #format_string, self.0)
            }
        }

        impl ::core::hash::Hash for #ident_id {
            #[inline]
            fn hash<H: ::core::hash::Hasher>(&self, state: &mut H) {
                state.#hash_write(self.0.get());
            }
        }
        // TODO: Maybe add this back later
        // impl ::nohash_hasher::IsEnabled for #ident_id {}

        impl #ident_id {
            #[doc = #ident_new_doc]
            #[inline]
            #vis_new fn new() -> Self {
                static NEXT_ID: #atomic_type = #atomic_type::new(1);

                let id_val = NEXT_ID.fetch_add(1, ::core::sync::atomic::Ordering::Relaxed);

                assert_ne!(0, id_val, #id_overflow_err);

                Self(#inner_type::new(id_val).unwrap())
            }
        }
    }
    .into()
}
