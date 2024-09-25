use proc_macro::TokenStream;
use queryable_tuples::make_combined_query_tuples_impl;

mod queryable_tuples;

#[proc_macro]
pub fn make_combined_query_tuples(input: TokenStream) -> TokenStream {
    make_combined_query_tuples_impl(input)
}
