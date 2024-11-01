//! Macros for users of WutEngine

#![allow(clippy::missing_docs_in_private_items)]

use quote::quote;

/// Implements the standard boilerplate required for an implementation of the `Component` trait.
/// Saves some typing
#[proc_macro]
pub fn component_boilerplate(_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    quote! {
        fn as_any(&self) -> &dyn ::std::any::Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn ::std::any::Any {
            self
        }
    }
    .into()
}
