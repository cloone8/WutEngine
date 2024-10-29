//! Internal macros for WutEngine. Not intended to be used by WutEngine users

#![allow(clippy::missing_docs_in_private_items)]

use quote::{format_ident, quote};
use syn::parse::Parse;
use syn::{parse_macro_input, Ident, Token};

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
