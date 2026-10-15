use core::iter::{Peekable, once};
use crate::{proc_macro::{Ident, Punct, PunctChar, TokenStream, TokenTree}, quote};

#[derive(Debug, Clone)]
pub enum GenericDefinition {
    Lifetime(Ident),
    Type(Ident, TokenStream),
}

#[derive(Debug, Clone, Default)]
pub struct GenericsDefinition(pub Vec<GenericDefinition>);

impl GenericsDefinition {
    pub fn parse(input: &mut Peekable<impl Iterator<Item = TokenTree>>) -> Self {
        let mut generics: Vec<GenericDefinition> = Vec::new();
        if let Some(TokenTree::Punct(punct)) = input.peek()
            && punct.char() == PunctChar::LessThan
        {
            let _ = input.next();

            loop {
                match input.next() {
                    Some(TokenTree::Ident(ident)) => {
                        let mut constraints = TokenStream::new();
                        if let Some(TokenTree::Punct(punct)) = input.peek()
                            && punct.char() == PunctChar::Colon
                        {
                            let _ = input.next();

                            let mut nested = 0;
                            while let Some(item) = input.peek() {
                                if let TokenTree::Punct(punct) = item {
                                    match punct.char() {
                                        PunctChar::LessThan => nested += 1,
                                        PunctChar::GreaterThan => {
                                            if nested == 0 {
                                                break;
                                            }
                                            nested -= 1;
                                        }
                                        PunctChar::Comma if nested == 0 => {
                                            let _ = input.next();
                                            break;
                                        }
                                        _ => {}
                                    }
                                }
                                constraints.extend(once(input.next().unwrap()));
                            }
                        }
                        generics.push(GenericDefinition::Type(ident, constraints));
                    }
                    Some(TokenTree::Punct(punct)) if punct.char() == PunctChar::GreaterThan => {
                        break;
                    }
                    Some(TokenTree::Punct(punct)) if punct.char() == PunctChar::Qoute => {
                        let lifetime_ident = match input.next() {
                            Some(TokenTree::Ident(ident)) => ident,
                            t => panic!("Expected lifetime name after `'`, found `{t:?}`"),
                        };
                        generics.push(GenericDefinition::Lifetime(lifetime_ident));

                        if let Some(TokenTree::Punct(punct)) = input.peek()
                            && punct.char() == PunctChar::Comma
                        {
                            let _ = input.next();
                        }
                    }
                    None => panic!("Expected Some after `<` in generics, found None"),
                    tt => panic!("Expected ident or `>` or \"'\" in generics, found `{tt:?}`"),
                }
            }
        }

        Self(generics)
    }
}

impl GenericsDefinition {
    /// produces the complete generic definition for an item, e.g. `<T, U>` or `<T: Clone>`.
    #[must_use]
    pub fn definition(&self) -> TokenStream {
        if self.0.is_empty() {
            return TokenStream::new();
        }

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
            .map(|generic| quote! {(@ generic.definition()),})
            .collect()
    }

    /// produces the usage of the generics for an item, e.g. `<T, U>` or `<T>`.
    #[must_use]
    pub fn usage(&self, exprs: Option<&[TokenStream]>) -> TokenStream {
        if self.0.is_empty() {
            return TokenStream::new();
        }

        quote! {
            < (@ self.generic_usage(exprs)) >
        }
    }

    /// produces the inner generic usages for an item without < and >, e.g. `T, U` or `T`.
    /// Always has a leading comma if there are any generic usages, so that it can be used in
    /// combination with other generic usages.
    #[must_use]
    #[allow(clippy::redundant_closure_for_method_calls)]
    pub fn generic_usage(&self, exprs: Option<&[TokenStream]>) -> TokenStream {
        assert!(
            self.0.len() >= exprs.map_or(0, |g| g.len()),
            "Tried to make usage of generics with too many generics"
        );

        let anonymous = quote! {(@ Ident::new("_"))};

        self.0
            .iter()
            .enumerate()
            .map(|(i, generic)| {
                quote! {(@ generic.usage(exprs.map(|g| g.get(i).unwrap_or(&anonymous)).cloned())),}
            })
            .collect()
    }
}

impl GenericDefinition {
    /// produces the complete generic definition for a single generic, e.g. `T` or `T: Clone`.
    #[must_use]
    pub fn definition(&self) -> TokenStream {
        match self {
            Self::Lifetime(ident) => TokenStream::from_iter([
                TokenTree::Punct(Punct::new(PunctChar::Qoute, true)),
                TokenTree::Ident(ident.clone()),
            ]),
            Self::Type(ident, bounds) => quote! {
                (@ ident.clone()) : (@ bounds.clone())
            },
        }
    }

    /// produces the usage of a single generic, e.g. `T` or `T` (without bounds).
    #[must_use]
    pub fn usage(&self, expr: Option<TokenStream>) -> TokenStream {
        match self {
            Self::Lifetime(ident) => {
                let mut tt =
                    TokenStream::from_iter([TokenTree::Punct(Punct::new(PunctChar::Qoute, true))]);
                tt.extend(expr.unwrap_or_else(|| quote! {(@ ident.clone())}));
                tt
            }
            Self::Type(ident, _) => expr.unwrap_or_else(|| quote! {(@ ident.clone())}),
        }
    }
}
