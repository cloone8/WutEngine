use proc_macro::TokenStream;
use queryable_tuples::make_queryable_tuples_impl;

mod queryable_tuples;

#[proc_macro]
pub fn make_queryable_tuples(input: TokenStream) -> TokenStream {
    make_queryable_tuples_impl(input)
}
