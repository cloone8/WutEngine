use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, punctuated::Punctuated, Ident, Token};

fn map_tokens<'a, T: Clone + 'static>(
    elems: impl IntoIterator<Item = &'a T>,
    func: impl Fn(T, usize) -> TokenStream,
) -> Punctuated<TokenStream, Token![,]> {
    map_tokens_punctuated(elems, func)
}

fn map_tokens_statements<'a, T: Clone + 'static>(
    elems: impl IntoIterator<Item = &'a T>,
    func: impl Fn(T, usize) -> proc_macro2::TokenStream,
) -> Punctuated<TokenStream, Token![;]> {
    map_tokens_punctuated(elems, func)
}

fn map_tokens_punctuated<'a, T: Clone + 'static, P: Default>(
    elems: impl IntoIterator<Item = &'a T>,
    func: impl Fn(T, usize) -> proc_macro2::TokenStream,
) -> Punctuated<TokenStream, P> {
    elems
        .into_iter()
        .cloned()
        .enumerate()
        .map(|(i, t)| func(t, i))
        .collect()
}

pub fn make_combined_query_tuples_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let idents = parse_macro_input!(input with Punctuated::<Ident, Token![,]>::parse_terminated);

    let wheres = map_tokens(&idents, |ident, _| {
        quote! {#ident: Queryable<'q>}
    });

    let type_ids = map_tokens(&idents, |ident, _| {
        quote! {::core::any::TypeId::of::<#ident::Inner>()}
    });

    let expected_cells = idents.len();

    let refs = map_tokens_statements(&idents, |ident, i| {
        let ident_ref = format_ident!("refs_{}", ident.to_string().to_lowercase());
        quote! {let mut #ident_ref = #ident::from_anyvec(cells[#i])}
    });

    let first_ref_ident = format_ident!(
        "refs_{}",
        idents.first().unwrap().to_string().to_lowercase()
    );

    let checks = map_tokens_statements(idents.iter().skip(1), |ident, _| {
        let ident_ref = format_ident!("refs_{}", ident.to_string().to_lowercase());
        quote! {debug_assert_eq!(#first_ref_ident.len(), #ident_ref.len())}
    });

    let all_refs_idents = map_tokens(&idents, |ident, _| {
        let ident_ref = format_ident!("refs_{}", ident.to_string().to_lowercase());
        quote! {#ident_ref}
    });

    quote! {
        impl<'q, #idents> CombinedQuery<'q> for (#idents)
        where
            #wheres
        {
            fn get_type_ids() -> Vec<::core::any::TypeId> {
                vec![
                    #type_ids
                ]
            }

            fn do_callback<Func, Out>(entities: &[EntityId], cells: Vec<&'q ::core::cell::UnsafeCell<AnyVec>>, callback: Func) -> Vec<Out> where Func: Fn(EntityId, Self) -> Out {
                assert_eq!(#expected_cells, cells.len());

                #refs;

                assert_eq!(entities.len(), #first_ref_ident.len());

                #checks;

                let mut outputs = Vec::with_capacity(#first_ref_ident.len());
                let combined = ::itertools::izip!(#all_refs_idents);

                for (args, &entity) in combined.zip(entities) {
                    outputs.push(callback(entity, args));
                }

                outputs
            }
        }
    }
    .into()
}
