#![doc = include_str!("../README.md")]
#![allow(
    clippy::missing_docs_in_private_items,
    reason = "A lot of useless boilerplate in macro types"
)]

use proc_macro::Span;
use quote::quote;
use syn::parse::Parse;
use syn::{Attribute, Ident, LitStr, Type, Visibility, parse_macro_input, parse_str};

/// Input for the [unique_id_type32] and [unique_id_type64] macros
struct UniqueIdTypeInput {
    /// Existing attributes to apply
    attrs: Vec<Attribute>,

    /// The visibility to assign to the generated type
    vis: Option<Visibility>,

    /// Name of the new ID type
    name: Ident,
}

impl Parse for UniqueIdTypeInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let vis: Option<Visibility> = input.parse::<Visibility>().ok();
        let name: Ident = input.parse()?;

        Ok(Self { attrs, vis, name })
    }
}

struct UniqueIdConfig {
    ident_id: Ident,
    attrs: Vec<Attribute>,
    vis: Option<Visibility>,
    atomic_type: Type,
    inner_type: Type,
    format_string: LitStr,
    hash_write: Ident,
}

/// Generates a new 32-bit atomic identifier type, which automatically increments itself
/// whenever a new instance is created.
#[proc_macro]
pub fn unique_id_type32(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as UniqueIdTypeInput);

    unique_id_type(UniqueIdConfig {
        ident_id: input.name,
        attrs: input.attrs,
        vis: input.vis,
        atomic_type: parse_str("::core::sync::atomic::AtomicU32").unwrap(),
        inner_type: parse_str("::core::num::NonZeroU32").unwrap(),
        format_string: LitStr::new("{:08x}", Span::call_site().into()),
        hash_write: parse_str("write_u32").unwrap(),
    })
}

/// Generates a new 64-bit atomic identifier type, which automatically increments itself
/// whenever a new instance is created.
#[proc_macro]
pub fn unique_id_type64(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as UniqueIdTypeInput);

    unique_id_type(UniqueIdConfig {
        ident_id: input.name,
        attrs: input.attrs,
        vis: input.vis,
        atomic_type: parse_str("::core::sync::atomic::AtomicU64").unwrap(),
        inner_type: parse_str("::core::num::NonZeroU64").unwrap(),
        format_string: LitStr::new("{:016x}", Span::call_site().into()),
        hash_write: parse_str("write_u64").unwrap(),
    })
}

fn unique_id_type(config: UniqueIdConfig) -> proc_macro::TokenStream {
    let ident_id = config.ident_id;

    let id_overflow_err = format!("Overflow for ID of type `{ident_id}`");

    let ident_new_doc = format!("Generate a new guaranteed unique [{ident_id}]");
    let attrs = config.attrs;
    let vis = config.vis;
    let vis_new = match &vis {
        Some(Visibility::Public(_)) => Some(parse_str("pub(crate)").unwrap()),
        Some(other) => Some(other.clone()),
        None => None,
    };
    let atomic_type = config.atomic_type;
    let inner_type = config.inner_type;
    let format_string = config.format_string;
    let hash_write = config.hash_write;

    quote! {
        #(#attrs)*
        #[repr(transparent)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
        #vis struct #ident_id(#inner_type);

        impl ::std::fmt::Display for #ident_id {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                write!(f, #format_string, self.0)
            }
        }

        impl ::core::hash::Hash for #ident_id {
            #[inline]
            fn hash<H: ::core::hash::Hasher>(&self, state: &mut H) {
                state.#hash_write(self.0.get());
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
            #vis_new fn new() -> Self {
                static NEXT_ID: #atomic_type = #atomic_type::new(1);

                let id_val = NEXT_ID.fetch_add(1, ::core::sync::atomic::Ordering::Relaxed);

                assert_ne!(0, id_val, #id_overflow_err);

                Self(#inner_type::new(id_val).unwrap())
            }
        }
    }
    .into()
}

/// Adds a `Self::VARIANT_COUNT` field containing the number of variants of the provided enum, with
/// the same visibility as the enum itself
#[proc_macro_derive(VariantCount)]
pub fn derive_variant_count(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let derive_input: syn::DeriveInput = syn::parse(input).unwrap();

    let ident = derive_input.ident;
    let generics = derive_input.generics.params;

    let len = match derive_input.data {
        syn::Data::Enum(enum_item) => enum_item.variants.len(),
        _ => panic!("VariantCount only works on Enums"),
    };

    let vis = derive_input.vis;

    let expanded = if generics.is_empty() {
        quote! {
            impl #ident {
                /// The amount of variants of [Self]
                #vis const VARIANT_COUNT: usize = #len;
            }
        }
    } else {
        quote! {
            impl<#generics> #ident<#generics> {
                /// The amount of variants of [Self]
                #vis const VARIANT_COUNT: usize = #len;
            }
        }
    };

    expanded.into()
}

/// Adds a `variant_name()` method that returns the name of the enum variant, with
/// the same visibility as the enum itself
#[proc_macro_derive(VariantName)]
pub fn derive_variant_name(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let derive_input: syn::DeriveInput = syn::parse(input).unwrap();

    let ident = derive_input.ident;
    let generics = derive_input.generics.params;

    let variants = match derive_input.data {
        syn::Data::Enum(enum_item) => enum_item.variants,
        _ => panic!("VariantCount only works on Enums"),
    };

    let mut variant_names = Vec::new();
    for variant in variants {
        let name = variant.ident.to_string();
        let ident = variant.ident;

        match variant.fields {
            syn::Fields::Named(_) => {
                variant_names.push(quote! { Self::#ident {..} => #name, });
            }
            syn::Fields::Unnamed(_) => {
                variant_names.push(quote! { Self::#ident(_) => #name, });
            }
            syn::Fields::Unit => {
                variant_names.push(quote! { Self::#ident => #name, });
            }
        }
    }

    let vis = derive_input.vis;

    let func_imp = quote! {
        /// The name of the variant of [Self] as a static [str]
        #[inline]
        #vis const fn variant_name(&self) -> &'static str {
            match self {
                #(#variant_names)*
            }
        }
    };

    let expanded = if generics.is_empty() {
        quote! {
            impl #ident {
                #func_imp
            }
        }
    } else {
        quote! {
            impl<#generics> #ident<#generics> {
                #func_imp
            }
        }
    };

    expanded.into()
}

/// Adds a `Self::variant_index` method containing the index of the variant of the enum, with
/// the same visibility as the enum itself
#[proc_macro_derive(VariantIndex, attributes(index_repr))]
pub fn derive_variant_index(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let derive_input: syn::DeriveInput = syn::parse(input).unwrap();

    let ident = derive_input.ident;
    let generics = derive_input.generics.params;

    let mut repr_type = None;

    for attr in derive_input.attrs {
        if let Some(ident) = attr.path().get_ident() {
            if ident == "index_repr" {
                repr_type = Some(attr.parse_args::<syn::Type>().unwrap());
                break;
            }
        }
    }

    let repr_type = repr_type.unwrap_or(syn::parse_str("u32").unwrap());

    let variants = match derive_input.data {
        syn::Data::Enum(enum_item) => enum_item.variants,
        _ => panic!("VariantIndex only works on Enums"),
    };

    let mut variant_indices = Vec::new();

    for (index, variant) in variants.into_iter().enumerate() {
        let ident = variant.ident;

        let index = syn::Index {
            index: index as u32,
            span: ident.span(),
        };

        match variant.fields {
            syn::Fields::Named(_) => {
                variant_indices.push(quote! { Self::#ident {..} => #index, });
            }
            syn::Fields::Unnamed(_) => {
                variant_indices.push(quote! { Self::#ident(_) => #index, });
            }
            syn::Fields::Unit => {
                variant_indices.push(quote! { Self::#ident => #index, });
            }
        }
    }

    let vis = derive_input.vis;

    let func_imp = quote! {
        /// The index of the variant of [Self]
        #[inline]
        #vis const fn variant_index(&self) -> #repr_type {
            match self {
                #(#variant_indices)*
            }
        }
    };

    let expanded = if generics.is_empty() {
        quote! {
            impl #ident {
                #func_imp
            }
        }
    } else {
        quote! {
            impl<#generics> #ident<#generics> {
                #func_imp
            }
        }
    };

    expanded.into()
}
