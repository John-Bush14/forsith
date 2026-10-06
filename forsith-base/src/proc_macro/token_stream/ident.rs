use crate::proc_macro::{TokenStream, TokenTree};

#[cfg(feature = "in_proc_macro")]
extern crate proc_macro;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ident(String);

#[cfg(feature = "in_proc_macro")]
impl From<proc_macro::Ident> for Ident {
    fn from(i: proc_macro::Ident) -> Self {
        Self(i.to_string())
    }
}

#[cfg(feature = "in_proc_macro")]
impl From<Ident> for proc_macro::Ident {
    fn from(i: Ident) -> Self {
        Self::new(&i.0, proc_macro::Span::call_site())
    }
}

impl From<String> for Ident {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl Extend<Ident> for TokenStream {
    fn extend<T: IntoIterator<Item = Ident>>(&mut self, iter: T) {
        self.0.extend(iter.into_iter().map(TokenTree::Ident));
    }
}

impl Ident {
    #[must_use]
    pub fn new(name: &str) -> Self {
        Self(name.to_string())
    }
}

impl core::fmt::Display for Ident {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl core::ops::Deref for Ident {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
