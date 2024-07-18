use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{parse::Parser, punctuated::Punctuated, token::Comma, Ident, Index, Token};

fn map_tokens<'a, T: Clone + 'static>(
    idents: impl IntoIterator<Item = &'a T>,
    func: impl Fn(T, usize) -> proc_macro2::TokenStream,
) -> Punctuated<proc_macro2::TokenStream, Token![,]> {
    idents
        .into_iter()
        .cloned()
        .enumerate()
        .map(|(i, t)| func(t, i))
        .collect()
}

fn map_tokens_statements<'a, T: Clone + 'static>(
    idents: impl IntoIterator<Item = &'a T>,
    func: impl Fn(T, usize) -> proc_macro2::TokenStream,
) -> Punctuated<proc_macro2::TokenStream, Token![;]> {
    idents
        .into_iter()
        .cloned()
        .enumerate()
        .map(|(i, t)| func(t, i))
        .collect()
}

fn map_tokens_append<'a, T: Clone + 'static>(
    idents: impl IntoIterator<Item = &'a T>,
    func: impl Fn(T, usize) -> proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    idents
        .into_iter()
        .cloned()
        .enumerate()
        .map(|(i, t)| func(t, i))
        .collect()
}

#[proc_macro]
pub fn generate_component_filter_for_tuple(input: TokenStream) -> TokenStream {
    let parsed = Punctuated::<Ident, Comma>::parse_separated_nonempty
        .parse(input)
        .unwrap();

    let idents: Vec<Ident> = parsed.clone().into_iter().collect();

    let mut idents_punctuated = Punctuated::<Ident, Token![,]>::new();
    idents_punctuated.extend(idents.clone());

    let trait_bounds = map_tokens(&idents, |ident, _| {
        quote! {#ident: wutengine_core::component::Component}
    });

    let ref_types = map_tokens(&idents, |ident, _| {
        quote! {&'a #ident}
    });

    let mut_types = map_tokens(&idents, |ident, _| {
        quote! {&'a mut #ident}
    });

    let component_ids = map_tokens(&idents, |ident, _| {
        quote! {#ident::COMPONENT_ID}
    });

    let arrs = map_tokens_statements(&idents, |ident, _| {
        let var_ident = format_ident!("{}_arr", ident.to_string().to_lowercase());
        quote! {let #var_ident = components.get(&#ident::COMPONENT_ID).expect(err_str)}
    });

    let first_ident = idents.first().cloned().unwrap();
    let first_arr = format_ident!("{}_arr", first_ident.to_string().to_lowercase());

    let entities = map_tokens_statements(&idents, |ident, _| {
        let arr_ident = format_ident!("{}_arr", ident.to_string().to_lowercase());
        let var_ident = format_ident!("{}_entities", ident.to_string().to_lowercase());
        quote! {let #var_ident = #arr_ident.get_multi::<#ident>(&entity_ids);}
    });

    let entities_mut = map_tokens_statements(&idents, |ident, _| {
        let arr_ident = format_ident!("{}_arr", ident.to_string().to_lowercase());
        let var_ident = format_ident!("{}_entities", ident.to_string().to_lowercase());
        quote! {let #var_ident = #arr_ident.get_mut_multi::<#ident>(&entity_ids);}
    });

    let entity_idents = map_tokens(&idents, |ident, _| {
        format_ident!("{}_entities", ident.to_string().to_lowercase()).into_token_stream()
    });

    let filters = map_tokens_append(&idents, |ident, idx| {
        let index: Index = Index {
            index: idx.try_into().unwrap(),
            span: ident.span(),
        };

        quote! {.filter(|components| components.#index.is_some())}
    });

    let maps = map_tokens(&idents, |ident, idx| {
        let index: Index = Index {
            index: idx.try_into().unwrap(),
            span: ident.span(),
        };

        quote! {components.#index.unwrap()}
    });

    let arr_ptrs = map_tokens_statements(&idents, |ident, _| {
        let ptr_ident = format_ident!("{}_arr_ptr", ident.to_string().to_lowercase());

        quote! {let #ptr_ident = components.get_mut(&#ident::COMPONENT_ID).expect(unknown_component_err_str) as *mut ComponentArray}
    });

    let arr_ptrs_as_mut = map_tokens_statements(&idents, |ident, _| {
        let arr_ident = format_ident!("{}_arr", ident.to_string().to_lowercase());
        let ptr_ident = format_ident!("{}_arr_ptr", ident.to_string().to_lowercase());

        quote! { let #arr_ident = unsafe { #ptr_ident.as_mut().expect(ptr_err_str)}}
    });

    let result = quote! {
        impl<#idents_punctuated> ComponentFilter for (#idents_punctuated)
        where
            #trait_bounds
        {
            type Output<'a> = Vec<(#ref_types)>;
            type OutputMut<'a> = Vec<(#mut_types)>;

            fn filter(components: &IntMap<ComponentTypeId, ComponentArray>) -> Self::Output<'_> {
                let err_str = "Unknown component type!";

                let component_types = vec![#component_ids];
                debug_assert_eq!(component_types.clone(), itertools::Itertools::unique(component_types.clone().into_iter()).collect::<Vec<_>>());

                #arrs;

                let entity_ids: Vec<EntityId> = #first_arr.slice::<#first_ident>().iter().map(|x| x.id).collect();

                unsafe {
                    #entities;

                    itertools::izip!(#entity_idents)
                        #filters
                        .map(|components| (#maps))
                        .collect()
                }
            }

            fn filter_mut(components: &mut IntMap<ComponentTypeId, ComponentArray>) -> Self::OutputMut<'_> {
                let unknown_component_err_str = "Unknown component type!";
                let ptr_err_str = "Pointer was just crated from valid reference!";

                let component_types = vec![#component_ids];
                debug_assert_eq!(component_types.clone(), itertools::Itertools::unique(component_types.clone().into_iter()).collect::<Vec<_>>());

                #arr_ptrs;

                #arr_ptrs_as_mut;

                let entity_ids: Vec<EntityId> = #first_arr.slice::<#first_ident>().iter().map(|x| x.id).collect();

                unsafe {
                    #entities_mut;

                    itertools::izip!(#entity_idents)
                        #filters
                        .map(|components| (#maps))
                        .collect()
                }
            }
        }
    };

    result.into()
}
