use componentfilter::make_componentfilter_tuples_impl;
use proc_macro::TokenStream;

mod componentfilter;

#[proc_macro]
pub fn make_componentfilter_tuples(input: TokenStream) -> TokenStream {
    make_componentfilter_tuples_impl(input)
}
