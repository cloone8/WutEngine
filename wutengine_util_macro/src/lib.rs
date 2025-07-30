//! Internal WutEngine macros

use quote::quote;
use syn::parse::Parse;
use syn::{Attribute, Ident, Visibility, parse_macro_input};

/// Input for the [unique_id_type] macro
struct UniqueIdTypeInput {
    /// Existing attributes to apply
    attrs: Vec<Attribute>,

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

/// Generates a new 64-bit atomic identifier type, which automatically increments itself
/// whenever a new instance is created.
#[proc_macro]
pub fn unique_id_type64(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as UniqueIdTypeInput);

    let ident_id = input.name;

    let id_overflow_err = format!("Overflow for ID of type `{ident_id}`");

    let ident_new_doc = format!("Generate a new guaranteed unique [{ident_id}]");
    let attrs = input.attrs;
    let vis = input.vis;

    quote! {
        #(#attrs)*
        #[repr(transparent)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
        #vis struct #ident_id(::core::num::NonZeroU64);

        impl ::std::fmt::Display for #ident_id {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                write!(f, "{:016x}", self.0)
            }
        }

        impl ::core::hash::Hash for #ident_id {
            fn hash<H: ::core::hash::Hasher>(&self, state: &mut H) {
                state.write_u64(self.0.get());
            }
        }

        impl ::wutengine_util::hash::nohash_hasher::IsEnabled for #ident_id {}

        impl Default for #ident_id {
            fn default() -> Self {
                Self::new()
            }
        }

        impl #ident_id {
            #[doc = #ident_new_doc]
            #[inline]
            #vis fn new() -> Self {
                static NEXT_ID: ::core::sync::atomic::AtomicU64 = ::core::sync::atomic::AtomicU64::new(1);

                let id_val = NEXT_ID.fetch_add(1, ::core::sync::atomic::Ordering::Relaxed);

                assert_ne!(0, id_val, #id_overflow_err);

                Self(::core::num::NonZeroU64::new(id_val).unwrap())
            }
        }
    }
    .into()
}

/// Generates a new 32-bit atomic identifier type, which automatically increments itself
/// whenever a new instance is created.
#[proc_macro]
pub fn unique_id_type32(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as UniqueIdTypeInput);

    let ident_id = input.name;

    let id_overflow_err = format!("Overflow for ID of type `{ident_id}`");

    let ident_new_doc = format!("Generate a new guaranteed unique [{ident_id}]");
    let attrs = input.attrs;
    let vis = input.vis;

    quote! {
        #(#attrs)*
        #[repr(transparent)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
        #vis struct #ident_id(::core::num::NonZeroU32);

        impl ::std::fmt::Display for #ident_id {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                write!(f, "{:08x}", self.0)
            }
        }

        impl ::core::hash::Hash for #ident_id {
            fn hash<H: ::core::hash::Hasher>(&self, state: &mut H) {
                state.write_u32(self.0.get());
            }
        }

        impl ::wutengine_util::hash::nohash_hasher::IsEnabled for #ident_id {}

        impl Default for #ident_id {
            fn default() -> Self {
                Self::new()
            }
        }

        impl #ident_id {
            #[doc = #ident_new_doc]
            #[inline]
            #vis fn new() -> Self {
                static NEXT_ID: ::core::sync::atomic::AtomicU32 = ::core::sync::atomic::AtomicU32::new(1);

                let id_val = NEXT_ID.fetch_add(1, ::core::sync::atomic::Ordering::Relaxed);

                assert_ne!(0, id_val, #id_overflow_err);

                Self(::core::num::NonZeroU32::new(id_val).unwrap())
            }
        }
    }
    .into()
}
