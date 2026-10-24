use crate::proc_macro::{Group, Ident, Literal, Punct, TokenStream, TokenTree};

#[macro_export]
macro_rules! quote {
    ($($tt:tt)*) => {{
        let mut tokens = $crate::proc_macro::TokenStream::new();
        $(
            tokens.extend($crate::quote_tree!($tt));
        )*
        tokens
    }};
}

#[macro_export]
macro_rules! quote_tree {
    ((@ $($tt:tt)*)) => {{
        let mut tokens = $crate::proc_macro::TokenStream::new();
        $crate::proc_macro::quote::ToTokens::to_tokens(($($tt)*), &mut tokens);
        tokens
    }};
    ($ident:ident) => {{
        [$crate::proc_macro::Ident::new(stringify!($ident))]
    }};
    (($($tt:tt)*)) => {{
        [$crate::proc_macro::Group::new($crate::proc_macro::Delimiter::Parenthesis, quote!($($tt)*))]
    }};
    ({$($tt:tt)*}) => {{
        [$crate::proc_macro::Group::new($crate::proc_macro::Delimiter::Brace, quote!($($tt)*))]
    }};
    ([$($tt:tt)*]) => {{
        [$crate::proc_macro::Group::new($crate::proc_macro::Delimiter::Bracket, quote!($($tt)*))]
    }};
    ($lit:literal) => {{
        [$crate::proc_macro::Literal::from($lit)]
    }};
    ($punct:tt) => {{
        let puncts = stringify!($punct);
        puncts
            .chars()
            .enumerate()
            .map(|(i, c)| {
                if c == '_' {
                    return $crate::proc_macro::TokenTree::Ident($crate::proc_macro::Ident::new("_"));
                }

                $crate::proc_macro::TokenTree::Punct($crate::proc_macro::Punct::new(c.try_into().expect("qoute! contained punctuation PunctChar can't contain?"), i != puncts.len() - 1))
            })
    }};
}

impl<T: Clone> ToTokens for T
where
    TokenStream: Extend<T>,
{
    fn to_tokens(self, ts: &mut TokenStream) {
        ts.extend(core::iter::once(self));
    }
}

macro_rules! impl_to_tokens_for_ref {
    ($($ty:ty),*) => {
        $(
            impl ToTokens for &$ty {
                fn to_tokens(self, ts: &mut TokenStream) {
                    self.clone().to_tokens(ts);
                }
            }
        )*
    };
}

impl_to_tokens_for_ref!(Ident, Literal, Punct, Group, TokenTree, TokenStream);

pub trait ToTokens {
    fn to_tokens(self, ts: &mut TokenStream);

    fn to_token_stream(self) -> TokenStream where Self: Sized {
        let mut ts = TokenStream::new();
        self.to_tokens(&mut ts);
        ts
    }
}
