//! Internal macros for WutEngine. Not intended to be used by WutEngine users

#![allow(clippy::missing_docs_in_private_items)]

use quote::{format_ident, quote};
use syn::parse::Parse;
use syn::{Attribute, Ident, Token, parse_macro_input};

struct GenerateAssertSetInput {
    name: Ident,
    _comma: Token![,],
    doc: syn::LitStr,
}

impl Parse for GenerateAssertSetInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            name: input.parse()?,
            _comma: input.parse()?,
            doc: input.parse()?,
        })
    }
}

/// Generates a set of assertions with the given name, gated by a feature derived by that name
#[proc_macro]
pub fn generate_assert_set(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let args = parse_macro_input!(input as GenerateAssertSetInput);

    let ident_normal = format_ident!("{}_assert", args.name);
    let ident_eq = format_ident!("{}_assert_eq", args.name);
    let ident_ne = format_ident!("{}_assert_ne", args.name);

    let feature_name = format_ident!("assert_{}", args.name);

    let doc = args.doc.value();
    let doc_feature = format!("Enabled only if feature `{}` is enabled.", feature_name);

    quote! {
        #[doc = #doc]
        /// Syntax matches [::core::assert]
        #[doc = #doc_feature]
        #[macro_export]
        macro_rules! #ident_normal {
            ($($arg:tt)*) => {
                if cfg!(feature = #feature_name) {
                    ::core::assert!($($arg)*)
                }
            };
        }

        #[doc = #doc]
        /// Syntax matches [::core::assert_eq]
        #[doc = #doc_feature]
        #[macro_export]
        macro_rules! #ident_eq {
            ($($arg:tt)*) => {
                if cfg!(feature = #feature_name) {
                    ::core::assert_eq!($($arg)*)
                }
            };
        }

        #[doc = #doc]
        /// Syntax matches [::core::assert_ne]
        #[doc = #doc_feature]
        #[macro_export]
        macro_rules! #ident_ne {
            ($($arg:tt)*) => {
                if cfg!(feature = #feature_name) {
                    ::core::assert_ne!($($arg)*)
                }
            };
        }

        pub use #ident_normal;
        pub use #ident_eq;
        pub use #ident_ne;
    }
    .into()
}

struct GenerateAtomicIdInput {
    attrs: Vec<Attribute>,
    name: Ident,
}

impl Parse for GenerateAtomicIdInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let name: Ident = input.parse()?;

        Ok(Self { attrs, name })
    }
}
/// Generates a new atomic identifier type, which automatically increments itself
/// whenever a new instance is created.
#[proc_macro]
pub fn generate_atomic_id(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as GenerateAtomicIdInput);

    let ident_id = input.name;

    let id_overflow_err = format!("Overflow for ID of type `{}`", ident_id);

    let ident_new_doc = format!("Generate a new guaranteed unique [{}]", ident_id);
    let attrs = input.attrs;
    quote! {
        #(#attrs)*
        #[repr(transparent)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct #ident_id(::core::num::NonZeroU64);

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

        impl ::nohash_hasher::IsEnabled for #ident_id {}

        impl Default for #ident_id {
            fn default() -> Self {
                Self::new()
            }
        }

        impl #ident_id {
            #[doc = #ident_new_doc]
            #[inline]
            pub fn new() -> Self {
                static NEXT_ID: ::core::sync::atomic::AtomicU64 = ::core::sync::atomic::AtomicU64::new(1);

                let mut id_val = 0;
                
                while id_val == 0 {
                    id_val = NEXT_ID.fetch_add(1, ::core::sync::atomic::Ordering::Relaxed);
                }

                debug_assert!(id_val < u64::MAX, #id_overflow_err);

                Self(::core::num::NonZeroU64::new(id_val).unwrap())
            }
        }
    }
    .into()
}
