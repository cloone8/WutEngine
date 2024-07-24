use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, punctuated::Punctuated, Ident, Index, Token};

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

fn map_tokens_append<'a, T: Clone + 'static>(
    elems: impl IntoIterator<Item = &'a T>,
    func: impl Fn(T, usize) -> proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    elems
        .into_iter()
        .cloned()
        .enumerate()
        .map(|(i, t)| func(t, i))
        .collect()
}

pub fn make_queryable_tuples_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let idents = parse_macro_input!(input with Punctuated::<Ident, Token![,]>::parse_terminated);

    let wheres = map_tokens(&idents, |ident, _| {
        quote! {#ident: Queryable<'a>}
    });

    let reads_fns = map_tokens_statements(&idents, |ident, _| {
        let ident_lower = format_ident!("{}", ident.to_string().to_lowercase());
        quote! {let mut #ident_lower = #ident::reads()}
    });

    let reads_caps: Punctuated<TokenStream, Token![+]> =
        map_tokens_punctuated(&idents, |ident, _| {
            let ident_lower = format_ident!("{}", ident.to_string().to_lowercase());
            quote! {#ident_lower.len()}
        });

    let reads_appends = map_tokens_statements(&idents, |ident, _| {
        let ident_lower = format_ident!("{}", ident.to_string().to_lowercase());
        quote! {reads.append(&mut #ident_lower)}
    });

    let writes_fns = map_tokens_statements(&idents, |ident, _| {
        let ident_lower = format_ident!("{}", ident.to_string().to_lowercase());
        quote! {let mut #ident_lower = #ident::writes()}
    });

    let writes_caps: Punctuated<TokenStream, Token![+]> =
        map_tokens_punctuated(&idents, |ident, _| {
            let ident_lower = format_ident!("{}", ident.to_string().to_lowercase());
            quote! {#ident_lower.len()}
        });

    let writes_appends = map_tokens_statements(&idents, |ident, _| {
        let ident_lower = format_ident!("{}", ident.to_string().to_lowercase());
        quote! {writes.append(&mut #ident_lower)}
    });

    let results = map_tokens_statements(&idents, |ident, _| {
        let result_ident = format_ident!("{}_results", ident.to_string().to_lowercase());

        quote! {let #result_ident = #ident::do_query(entities, components)}
    });

    let len_asserts = map_tokens_statements(&idents, |ident, _| {
        let result_ident = format_ident!("{}_results", ident.to_string().to_lowercase());

        quote! {debug_assert_eq!(entity_len, #result_ident.len());}
    });

    let comps_only = map_tokens_statements(&idents, |ident, _| {
        let result_ident = format_ident!("{}_results", ident.to_string().to_lowercase());
        let comps_only_ident = format_ident!("{}_comps_only", ident.to_string().to_lowercase());

        quote! {let #comps_only_ident: Vec<Option<#ident>> = #result_ident.into_iter().map(|(_, comp)| comp).collect()}
    });

    let comps_only_idents: Punctuated<Ident, Token![,]> = idents
        .iter()
        .map(|ident| format_ident!("{}_comps_only", ident.to_string().to_lowercase()))
        .collect();

    let some_checks: Punctuated<TokenStream, Token![&&]> =
        map_tokens_punctuated(&idents, |ident, idx| {
            let index = Index {
                index: idx as u32,
                span: ident.span(),
            };

            quote! {opts.#index.is_some()}
        });

    let unwraps = map_tokens(&idents, |ident, idx| {
        let index = Index {
            index: idx as u32,
            span: ident.span(),
        };

        quote! {opts.#index.unwrap()}
    });

    quote! {
        unsafe impl<'a, #idents> Queryable<'a> for (#idents)
        where
            #wheres
        {
            fn reads() -> Vec<ComponentTypeId> {
                #reads_fns;

                let mut reads = Vec::with_capacity(#reads_caps);

                #reads_appends;

                reads
            }

            fn writes() -> Vec<ComponentTypeId> {
                #writes_fns;

                let mut writes = Vec::with_capacity(#writes_caps);

                #writes_appends;

                writes
            }

            fn do_query(
                entities: &'a [EntityId],
                components: &'a IntMap<ComponentTypeId, UnsafeCell<ComponentStorage>>,
            ) -> Vec<(EntityId, Option<Self>)> {
                let entity_len = entities.len();

                #results;

                #len_asserts;

                #comps_only;

                entities
                    .iter()
                    .copied()
                    .zip(itertools::izip!(#comps_only_idents))
                    .map(|(id, opts)| {
                        let all_some = #some_checks;

                        match all_some {
                            true => (
                                id,
                                Some((#unwraps)),
                            ),
                            false => (id, None),
                        }
                    })
                    .collect()
            }
        }
    }
    .into()
}
