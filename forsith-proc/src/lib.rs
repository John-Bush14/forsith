#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

extern crate proc_macro;
use proc_macro::TokenStream;

mod derive_more;

mod paste;

macro_rules! derive_macros {
    ($($name:ident $(#[$($att:ident),+ $(,)?])? => $func:path ),* $(,)?) => {
        $(
            #[allow(non_snake_case)]
            #[proc_macro_derive($name $(, attributes($($att),+))? )]
            pub fn $name(input: TokenStream) -> TokenStream {
                $func(input.into()).into()
            }
        )*
    };
}

derive_macros! {
    IsVariant => derive_more::derive_is_variant,
    Deref #[deref] => derive_more::derive_deref,
    DerefMut #[deref_mut] => derive_more::derive_deref_mut,
    Index #[index] => derive_more::derive_index,
    IndexMut #[index_mut] => derive_more::derive_index_mut,
    FromNum #[default] => derive_more::derive_from_num,
    TryFromNum => derive_more::derive_try_from_num,
    IntoNum => derive_more::derive_into_num,
    TryIntoNum => derive_more::derive_try_into_num,
    Not #[not] => derive_more::derive_not,
    Display #[display] => derive_more::Display,
    Octal #[octal] => derive_more::Octal,
    LowerHex #[lower_hex] => derive_more::LowerHex,
    UpperHex #[upper_hex] => derive_more::UpperHex,
    Binary #[binary] => derive_more::Binary,
    Pointer #[pointer] => derive_more::Pointer,
    LowerExp #[lower_exp] => derive_more::LowerExp,
    UpperExp #[upper_exp] => derive_more::UpperExp,
    Error #[from] => derive_more::derive_error,
}

#[proc_macro]
pub fn paste(input: TokenStream) -> TokenStream {
    paste::paste(input.into()).into()
}
