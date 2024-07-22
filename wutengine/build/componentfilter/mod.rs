use itertools::{repeat_n, Itertools};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{punctuated::Punctuated, Ident, Index, Token};

fn map_tokens<'a, T: Clone + 'static>(
    elems: impl IntoIterator<Item = &'a T>,
    func: impl Fn(T, usize) -> TokenStream,
) -> Punctuated<TokenStream, Token![,]> {
    elems
        .into_iter()
        .cloned()
        .enumerate()
        .map(|(i, t)| func(t, i))
        .collect()
}

fn map_tokens_statements<'a, T: Clone + 'static>(
    elems: impl IntoIterator<Item = &'a T>,
    func: impl Fn(T, usize) -> proc_macro2::TokenStream,
) -> Punctuated<TokenStream, Token![;]> {
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

#[derive(Debug, Clone)]
struct ComponentConfig {
    name: Ident,
    is_mut: bool,
    is_opt: bool,
}

fn make_configs_for_ident(ident: Ident) -> [ComponentConfig; 4] {
    let cfg_ref = ComponentConfig {
        name: ident.clone(),
        is_mut: false,
        is_opt: false,
    };

    let cfg_mut = ComponentConfig {
        name: ident.clone(),
        is_mut: true,
        is_opt: false,
    };

    let cfg_ref_opt = ComponentConfig {
        name: ident.clone(),
        is_mut: false,
        is_opt: true,
    };

    let cfg_mut_opt = ComponentConfig {
        name: ident.clone(),
        is_mut: true,
        is_opt: true,
    };

    [cfg_ref, cfg_mut, cfg_ref_opt, cfg_mut_opt]
}

fn make_componentfilter_impl(configs: Vec<ComponentConfig>) -> TokenStream {
    let types: Punctuated<Ident, Token![,]> = configs.iter().cloned().map(|c| c.name).collect();

    let impl_types = map_tokens(&configs, |c, _| {
        let name = c.name;

        if c.is_opt {
            if c.is_mut {
                quote! {Option<&'a mut #name>}
            } else {
                quote! {Option<&'a #name>}
            }
        } else if c.is_mut {
            quote! {&'a mut #name}
        } else {
            quote! {&'a #name}
        }
    });

    let wheres = map_tokens(&configs, |c, _| {
        let name = c.name;

        quote! {#name: wutengine_core::component::Component}
    });

    let output_types = map_tokens(&configs, |c, _| {
        let name = c.name;

        if c.is_opt {
            if c.is_mut {
                quote! {Option<&'o mut #name>}
            } else {
                quote! {Option<&'o #name>}
            }
        } else if c.is_mut {
            quote! {&'o mut #name}
        } else {
            quote! {&'o #name}
        }
    });

    let refcells = map_tokens_statements(&configs, |c, _| {
        let type_ident = c.name.clone();
        let refcell_ident = format_ident!("{}_refcell", c.name.to_string().to_ascii_lowercase());

        quote! { let #refcell_ident = components.get(&#type_ident::COMPONENT_ID).unwrap() }
    });

    let storages = map_tokens_statements(&configs, |c, _| {
        let refcell_ident = format_ident!("{}_refcell", c.name.to_string().to_ascii_lowercase());
        let storage_ident = format_ident!("{}", c.name.to_string().to_ascii_lowercase());

        if c.is_mut {
            quote! { let mut #storage_ident = #refcell_ident.borrow_mut() }
        } else {
            quote! { let #storage_ident = #refcell_ident.borrow() }
        }
    });

    let components = map_tokens_statements(&configs, |c, _| {
        let type_ident = c.name.clone();
        let storage_ident = format_ident!("{}", c.name.to_string().to_ascii_lowercase());
        let components_ident =
            format_ident!("{}_components", c.name.to_string().to_ascii_lowercase());

        if c.is_mut {
            quote! { let #components_ident = unsafe { #storage_ident.get_mut_multi::<#type_ident>(entities) } }
        } else {
            quote! { let #components_ident = unsafe { #storage_ident.get_multi::<#type_ident>(entities) } }
        }
    });

    let components_idents: Punctuated<Ident, Token![,]> = configs
        .iter()
        .map(|c| format_ident!("{}_components", c.name.to_string().to_ascii_lowercase()))
        .collect();

    let filters: TokenStream = configs
        .iter()
        .enumerate()
        .filter(|(_, c)| !c.is_opt)
        .map(|(idx, c)| {
            let index: Index = Index {
                index: idx as u32,
                span: c.name.span(),
            };

            quote! { .filter(|(_, components)| components.#index.is_some())}
        })
        .collect();

    let maps = map_tokens(&configs, |c, i| {
        let index: Index = Index {
            index: i as u32,
            span: c.name.span(),
        };

        if c.is_opt {
            quote! { components.#index }
        } else {
            quote! { components.#index.unwrap() }
        }
    });

    quote! {
        impl<'a, #types> crate::world::ComponentFilter<'a> for (#impl_types)
        where
            #wheres,
        {
            type Output<'o> = (#output_types);

            fn filter<FType>(
                entities: &[wutengine_core::entity::EntityId],
                components: &'a nohash_hasher::IntMap<wutengine_core::component::ComponentTypeId, core::cell::RefCell<crate::component::storage::ComponentStorage>>,
                func: FType,
            ) where
                FType: for<'x> FnOnce(Vec<(wutengine_core::entity::EntityId, Self::Output<'x>)>),
            {
                #refcells;

                #storages;

                #components;

                let result = entities
                    .iter()
                    .copied()
                    .zip(itertools::izip!(#components_idents))
                    #filters
                    .map(|(id, components)| (id, (#maps)))
                    .collect();

                func(result);
            }
        }
    }
}

fn make_componentfilter_tuple(len: u64) -> TokenStream {
    let mut output = TokenStream::new();

    let mut element_configs: Vec<[ComponentConfig; 4]> = Vec::with_capacity(len as usize);

    for i in 0..len {
        let ident = format_ident!("{}", char::from_u32(0x41 + (i as u32)).unwrap());
        element_configs.push(make_configs_for_ident(ident));
    }

    for config_idxs in repeat_n(0..4, len as usize).multi_cartesian_product() {
        let configs: Vec<ComponentConfig> = config_idxs
            .into_iter()
            .enumerate()
            .map(|(i, idx)| element_configs[i][idx].clone())
            .collect();

        output.extend(make_componentfilter_impl(configs));
    }

    output
}

pub fn make_componentfilter_tuples(len: usize) -> TokenStream {
    let mut output = TokenStream::new();

    for i in 2..=len {
        output.extend(make_componentfilter_tuple(i as u64));
    }

    output
}
