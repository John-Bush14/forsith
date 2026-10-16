use crate::{
    proc_macro::{
        Delimiter, Ident, Punct, PunctChar, TokenStream, TokenTree, items::parsing::parse_type,
    },
    quote,
};
use core::iter::{Peekable, once};

#[derive(Debug, Clone)]
pub enum GenericParam {
    Lifetime(Ident),
    Type(Ident, TokenStream),
}

impl GenericParam {
    fn parse_type(stream: &mut Peekable<impl Iterator<Item = TokenTree>>, ident: Ident) -> Self {
        Self::Type(
            ident,
            match stream.peek() {
                Some(tt) if tt.is_punct(PunctChar::Colon) => {
                    let _ = stream.next();
                    parse_type::<false, true, false, false, true>(stream)
                }
                _ => TokenStream::new(),
            },
        )
    }

    fn parse_lifetime(stream: &mut Peekable<impl Iterator<Item = TokenTree>>) -> Self {
        Self::Lifetime(
            stream
                .next()
                .expect("Expected lifetime after `'`")
                .ident()
                .expect("Expected identifier after `'`"),
        )
    }
}

#[derive(Debug, Clone)]
pub struct WhereClause(pub TokenStream, pub TokenStream);

impl WhereClause {
    pub fn parse(input: &mut Peekable<impl Iterator<Item = TokenTree>>) -> Self {
        let s = Self(
            parse_type::<true, false, true, false, false>(input),
            parse_type::<false, true, false, true, false>(input),
        );

        if input.peek().is_some_and(|tt| tt.is_punct(PunctChar::Comma)) {
            let _ = input.next();
        }

        s
    }

    #[must_use]
    pub fn definition(&self) -> TokenStream {
        quote! {
            (@ self.0.clone()) : (@ self.1.clone())
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct GenericsDefinition {
    pub params: Vec<GenericParam>,
    pub where_clauses: Vec<WhereClause>,
}

impl GenericsDefinition {
    pub fn parse_params(ts: &mut Peekable<impl Iterator<Item = TokenTree>>) -> Vec<GenericParam> {
        let mut params = Vec::new();

        if ts.peek().is_some_and(|tt| tt.is_punct(PunctChar::LessThan)) {
            let _ = ts.next();
        } else {
            return params;
        }

        #[allow(irrefutable_let_patterns)]
        while let token = ts.next().expect("Generic param list ended unexpectedly") {
            match token {
                TokenTree::Ident(ident) => {
                    params.push(GenericParam::parse_type(ts, ident));
                }
                TokenTree::Punct(punct) if punct.char() == PunctChar::Qoute => {
                    params.push(GenericParam::parse_lifetime(ts));
                }
                TokenTree::Punct(punct) if punct.char() == PunctChar::Comma => (),
                TokenTree::Punct(punct) if punct.char() == PunctChar::GreaterThan => {
                    break;
                }
                tt => panic!(
                    "Expected ident, \"'\" or `>` in generics, found `{tt:?}`, current_context: `{params:?}`"
                ),
            }
        }

        params
    }

    pub fn parse_where_clauses(
        stream: &mut Peekable<impl Iterator<Item = TokenTree>>,
    ) -> Vec<WhereClause> {
        let mut where_clauses = Vec::new();

        if stream.peek().is_some_and(|tt| tt.is_ident("where")) {
            while let tt = stream
                .peek()
                .expect("Expected where clause to end with `;` or `{`")
                && !(tt.is_punct(PunctChar::Semicolon) || tt.is_group(Delimiter::Brace))
            {
                where_clauses.push(WhereClause::parse(stream));
            }
        }

        where_clauses
    }

    #[must_use]
    pub const fn assemble(params: Vec<GenericParam>, where_clauses: Vec<WhereClause>) -> Self {
        Self {
            params,
            where_clauses,
        }
    }

    /// produces the complete where clause definition for an item, e.g. `where T: Clone, U: Debug`.
    #[must_use]
    pub fn where_clause_definition(&self) -> TokenStream {
        if self.where_clauses.is_empty() {
            return TokenStream::new()
        }

        quote! {
            where (@ self.inner_where_clause_definition())
        }
    }

    /// produces the inner where clause definitions for an item without `where`, e.g. `T: Clone, U:
    /// Debug`.
    #[must_use]
    pub fn inner_where_clause_definition(&self) -> TokenStream {
        self.where_clauses
            .iter()
            .map(|where_clause| quote! {(@ where_clause.definition()),})
            .collect()
    }

    /// produces the complete generic param definition for an item, e.g. `<T, U>` or `<T: Clone>`.
    #[must_use]
    pub fn param_definition(&self) -> TokenStream {
        quote! {
            < (@ self.inner_param_definitions()) >
        }
    }

    /// produces the inner generic param definitions for an item without < and >, e.g. `T, U` or `T: Clone`.
    /// Always has a leading comma if there are any generic definitions, so that it can be used in
    /// combination with other generic definitions.
    #[must_use]
    pub fn inner_param_definitions(&self) -> TokenStream {
        self.params
            .iter()
            .map(|generic| quote! {(@ generic.definition()),})
            .collect()
    }

    /// produces the usage of the generics for an item, e.g. `<T, U>` or `<T>`.
    #[must_use]
    pub fn param_usage(&self, exprs: Option<&[TokenStream]>) -> TokenStream {
        quote! {
            < (@ self.inner_param_usage(exprs)) >
        }
    }

    /// produces the inner generic usages for an item without < and >, e.g. `T, U` or `T`.
    /// Always has a leading comma if there are any generic usages, so that it can be used in
    /// combination with other generic usages.
    #[must_use]
    #[allow(clippy::redundant_closure_for_method_calls)]
    pub fn inner_param_usage(&self, exprs: Option<&[TokenStream]>) -> TokenStream {
        assert!(
            self.params.len() >= exprs.map_or(0, |g| g.len()),
            "Tried to make usage of generics with too many generics"
        );

        let anonymous = quote! {(@ Ident::new("_"))};

        self.params
            .iter()
            .enumerate()
            .map(|(i, generic)| {
                quote! {(@ generic.usage(exprs.map(|g| g.get(i).unwrap_or(&anonymous)).cloned())),}
            })
            .collect()
    }
}

impl GenericParam {
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
