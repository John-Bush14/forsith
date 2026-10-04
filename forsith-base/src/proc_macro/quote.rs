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
    ((@ $($tt:tt)*)) => {[$($tt)*]};
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
            .map(|(i, c)| $crate::proc_macro::Punct::new(c.try_into().expect("qoute! contained punctuation PunctChar can't contain?"), i != puncts.len() - 1))
    }};
}
