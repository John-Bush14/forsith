use core::{
    fmt::Display,
    ops::{Index, IndexMut},
};
use crate::{alloc::{
    string::{String, ToString},
    vec::Vec,
}, proc_macro::token_stream::{group::Group, ident::Ident, literal::Literal, punct::Punct}};

#[cfg(feature = "in_proc_macro")]
extern crate proc_macro;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TokenStream(Vec<TokenTree>);

pub mod ident;

pub mod punct;

pub mod group;

pub mod literal;

impl TokenStream {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        self.0.len()
    }

    fn raw_code(&self) -> String {
        self.0.iter().fold(String::new(), |mut code, tt| {
            if !matches!(tt, TokenTree::Punct(p) if !p.joint()) {
                code.push(' ');
            }

            code.push_str(&tt.to_string());
            code
        })
    }

    #[must_use]
    pub fn formatted_code(&self) -> String {
        let raw_code = self.raw_code();
        Self::format_code(&raw_code).unwrap_or(raw_code)
    }

    fn format_code(raw_code: &str) -> Option<String> {
        #[cfg(feature = "std")]
        {
            use std::{
                io::{Read, Write},
                   process,
            };

            let mut output = String::new();

            let proc = process::Command::new("rustfmt")
                .stdin(process::Stdio::piped())
                .stdout(process::Stdio::piped())
                .spawn().ok()?;

            let mut stdin = proc.stdin?;
            let mut stdout = proc.stdout?;

            stdin.write_all(raw_code.as_bytes()).ok()?;
            drop(stdin);
            stdout.read_to_string(&mut output).ok()?;

            Some(output)
        }

        #[cfg(not(feature = "std"))]
        let _ = raw_code;
        #[cfg(not(feature = "std"))]
        None
    }


}

impl Index<usize> for TokenStream {
    type Output = TokenTree;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<usize> for TokenStream {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

#[cfg(feature = "in_proc_macro")]
impl From<proc_macro::TokenStream> for TokenStream {
    fn from(ts: proc_macro::TokenStream) -> Self {
        Self(ts.into_iter().map(Into::into).collect())
    }
}

#[cfg(feature = "in_proc_macro")]
impl From<TokenStream> for proc_macro::TokenStream {
    fn from(sts: TokenStream) -> Self {
        let mut pcts = Self::new();
        pcts.extend(sts.0.into_iter().map(Into::<proc_macro::TokenTree>::into));
        pcts
    }
}

impl From<Vec<TokenTree>> for TokenStream {
    fn from(value: Vec<TokenTree>) -> Self {
        Self(value)
    }
}

impl Display for TokenStream {
    #[allow(clippy::unit_cmp)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.formatted_code())
    }
}

impl FromIterator<Self> for TokenStream {
    fn from_iter<I: IntoIterator<Item = Self>>(iter: I) -> Self {
        let mut ts = Self::default();
        ts.extend(iter);
        ts
    }
}

impl IntoIterator for TokenStream {
    type Item = TokenTree;
    type IntoIter = alloc::vec::IntoIter<TokenTree>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Extend<Self> for TokenStream {
    fn extend<I: IntoIterator<Item = Self>>(&mut self, iter: I) {
        for ts in iter {
            self.0.extend(ts);
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TokenTree {
    Group(Group),
    Ident(Ident),
    Punct(Punct),
    Literal(Literal),
}

#[cfg(feature = "in_proc_macro")]
impl From<proc_macro::TokenTree> for TokenTree {
    fn from(tt: proc_macro::TokenTree) -> Self {
        match tt {
            proc_macro::TokenTree::Group(g) => Self::Group(g.into()),
            proc_macro::TokenTree::Ident(i) => Self::Ident(i.into()),
            proc_macro::TokenTree::Punct(p) => Self::Punct(p.into()),
            proc_macro::TokenTree::Literal(l) => Self::Literal(l.into()),
        }
    }
}

#[cfg(feature = "in_proc_macro")]
impl From<TokenTree> for proc_macro::TokenTree {
    fn from(tt: TokenTree) -> Self {
        match tt {
            TokenTree::Group(g) => Self::Group(g.into()),
            TokenTree::Ident(i) => Self::Ident(i.into()),
            TokenTree::Punct(p) => Self::Punct(p.into()),
            TokenTree::Literal(l) => Self::Literal(l.into()),
        }
    }
}

impl Display for TokenTree {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Group(g) => write!(f, "{g}"),
            Self::Ident(i) => write!(f, "{i}"),
            Self::Punct(p) => write!(f, "{p}"),
            Self::Literal(l) => write!(f, "{l}"),
        }
    }
}

impl Extend<TokenTree> for TokenStream {
    fn extend<T: IntoIterator<Item = TokenTree>>(&mut self, iter: T) {
        self.0.extend(iter);
    }
}

impl FromIterator<TokenTree> for TokenStream {
    fn from_iter<I: IntoIterator<Item = TokenTree>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}
