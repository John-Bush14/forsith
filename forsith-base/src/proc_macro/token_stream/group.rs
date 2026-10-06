#[cfg(feature = "in_proc_macro")]
extern crate proc_macro;
use crate::proc_macro::{TokenStream, TokenTree};

#[derive(Clone, Debug, PartialEq)]
pub struct Group {
    pub delimiter: Delimiter,
    pub stream: TokenStream,
}

#[cfg(feature = "in_proc_macro")]
impl From<proc_macro::Group> for Group {
    fn from(g: proc_macro::Group) -> Self {
        Self {
            delimiter: g.delimiter().into(),
            stream: g.stream().into(),
        }
    }
}

#[cfg(feature = "in_proc_macro")]
impl From<Group> for proc_macro::Group {
    fn from(g: Group) -> Self {
        let mut group = Self::new(g.delimiter().into(), g.stream.into());
        group.set_span(proc_macro::Span::call_site());
        group
    }
}

impl core::fmt::Display for Group {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let (open, close) = match self.delimiter {
            Delimiter::Parenthesis => ('(', ')'),
            Delimiter::Brace => ('{', '}'),
            Delimiter::Bracket => ('[', ']'),
            Delimiter::None => (' ', ' '),
        };
        write!(f, "{}{}{}", open, self.stream.raw_code(), close)
    }
}

impl Extend<Group> for TokenStream {
    fn extend<T: IntoIterator<Item = Group>>(&mut self, iter: T) {
        self.0.extend(iter.into_iter().map(TokenTree::Group));
    }
}

impl Group {
    #[must_use]
    pub const fn new(delimiter: Delimiter, stream: TokenStream) -> Self {
        Self { delimiter, stream }
    }

    #[must_use]
    pub fn decompose(self) -> (Delimiter, TokenStream) {
        (self.delimiter, self.stream)
    }
    #[must_use]
    pub const fn stream(&self) -> &TokenStream {
        &self.stream
    }
    #[must_use]
    pub fn take_stream(self) -> TokenStream {
        self.stream
    }
    #[must_use]
    pub const fn delimiter(&self) -> Delimiter {
        self.delimiter
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub enum Delimiter {
    Parenthesis,
    Brace,
    Bracket,
    None,
}

#[cfg(feature = "in_proc_macro")]
impl From<proc_macro::Delimiter> for Delimiter {
    fn from(d: proc_macro::Delimiter) -> Self {
        match d {
            proc_macro::Delimiter::Parenthesis => Self::Parenthesis,
            proc_macro::Delimiter::Brace => Self::Brace,
            proc_macro::Delimiter::Bracket => Self::Bracket,
            proc_macro::Delimiter::None => Self::None,
        }
    }
}

#[cfg(feature = "in_proc_macro")]
impl From<Delimiter> for proc_macro::Delimiter {
    fn from(d: Delimiter) -> Self {
        match d {
            Delimiter::Parenthesis => Self::Parenthesis,
            Delimiter::Brace => Self::Brace,
            Delimiter::Bracket => Self::Bracket,
            Delimiter::None => Self::None,
        }
    }
}
