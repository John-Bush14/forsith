use crate::{proc_macro::{Punct, PunctChar, TokenStream, TokenTree, items::{GenericDefinition, GenericsDefinition, ItemDefinition}}, quote};

impl GenericsDefinition {
    /// produces the complete generic definition for an item, e.g. `<T, U>` or `<T: Clone>`.
    #[must_use]
    pub fn definition(&self) -> TokenStream {
        if self.0.is_empty() {return TokenStream::new();}

        quote! {
            < (@ self.generic_definitions()) >
        }
    }

    /// produces the inner generic definitions for an item without < and >, e.g. `T, U` or `T: Clone`.
    /// Always has a leading comma if there are any generic definitions, so that it can be used in
    /// combination with other generic definitions.
    #[must_use]
    pub fn generic_definitions(&self) -> TokenStream {
        self.0
            .iter()
            .map(|generic| quote!{(@ generic.definition()),})
            .collect()
    }

    /// produces the usage of the generics for an item, e.g. `<T, U>` or `<T>`.
    #[must_use]
    pub fn usage(&self) -> TokenStream {
        if self.0.is_empty() {return TokenStream::new();}

        quote! {
            < (@ self.generic_usage()) >
        }
    }

    /// produces the inner generic usages for an item without < and >, e.g. `T, U` or `T`.
    /// Always has a leading comma if there are any generic usages, so that it can be used in
    /// combination with other generic usages.
    #[must_use]
    pub fn generic_usage(&self) -> TokenStream {
        self.0
            .iter()
            .map(|generic| quote!{(@ generic.usage()),})
            .collect()
    }
}

impl GenericDefinition {
    /// produces the complete generic definition for a single generic, e.g. `T` or `T: Clone`.
    #[must_use]
    pub fn definition(&self) -> TokenStream {
        match self {
            Self::Lifetime(ident) => {
                TokenStream::from_iter([
                    TokenTree::Punct(Punct::new(PunctChar::Qoute, true)),
                    TokenTree::Ident(ident.clone()),
                ])
            },
            Self::Type(ident, bounds) => quote!{
                (@ ident.clone()) : (@ bounds.clone())
            },
        }
    }

    /// produces the usage of a single generic, e.g. `T` or `T` (without bounds).
    #[must_use]
    pub fn usage(&self) -> TokenStream {
        match self {
            Self::Lifetime(ident) => TokenStream::from_iter([
                TokenTree::Punct(Punct::new(PunctChar::Qoute, true)),
                TokenTree::Ident(ident.clone()),
            ]),
            Self::Type(ident, _) => quote!{
                (@ ident.clone())
            }
        }
    }
}

pub fn impl_item(item: &impl ItemDefinition, r#trait: Option<TokenStream>, body: TokenStream) -> TokenStream {
    let generics = item.generics();

    let r#trait = r#trait.map_or_default(|t| quote!{ (@ t) for });

    quote!{
        impl (@ generics.definition()) (@ r#trait) (@ item.name().clone()) (@ generics.usage()) {
            (@ body)
        }
    }
}
