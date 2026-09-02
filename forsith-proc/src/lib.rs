extern crate proc_macro;
use proc_macro::TokenStream;

#[macro_use]
pub(crate) mod utils;

mod derive_more;

#[proc_macro_derive(IsVariant)]
pub fn derive_is_variant(input: TokenStream) -> TokenStream {
    derive_more::derive_is_variant(input)
}
