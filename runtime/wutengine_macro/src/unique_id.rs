//! Generating unique IDs

use quote::quote;
use syn::Attribute;
use syn::Ident;
use syn::LitStr;
use syn::Type;
use syn::Visibility;

/// Configuration used when calling [`unique_id_type`]
pub(crate) struct UniqueIdConfig {
    /// The identifier for the new ID
    pub(crate) ident_id: Ident,

    /// Any attributes to pass through
    pub(crate) attrs: Vec<Attribute>,

    /// The visibility of the identifier
    pub(crate) vis: Option<Visibility>,

    /// What type of atomic generator to use
    pub(crate) atomic_type: Type,

    /// The actual inner type of the ID
    pub(crate) inner_type: Type,

    /// The formatting string to use for the `Display` trait
    pub(crate) format_string: LitStr,

    /// The name of the hash-write function on a `Hasher`
    pub(crate) hash_write: Ident,
}

/// Generate the code for a new unqiue ID according to the given `config`
pub(crate) fn generate(config: UniqueIdConfig) -> proc_macro::TokenStream {
    let ident_id = config.ident_id;

    let id_overflow_err = format!("Overflow for ID of type `{ident_id}`");

    let ident_new_doc = format!("Generate a new guaranteed unique [{ident_id}]");
    let attrs = config.attrs;
    let vis = config.vis;
    let vis_new = match &vis {
        Some(Visibility::Public(_)) => {
            Some(syn::parse_str("pub(crate)").expect("Invalid str given"))
        }
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
        // TODO: Maybe add this back later
        // impl ::nohash_hasher::IsEnabled for #ident_id {}

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
