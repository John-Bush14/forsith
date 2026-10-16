use forsith_base::{
    casing::{Casing, change_casing},
    proc_macro::{
        Ident, TokenStream,
        items::{
            generating::impl_item,
            parsing::{DeriveInput, parse_derive_input},
        },
    },
    quote,
};

mod convert;
pub use convert::{derive_from_num, derive_into_num, derive_try_from_num, derive_try_into_num};

mod ssfd;
pub use ssfd::{derive_deref, derive_deref_mut, derive_index, derive_index_mut, derive_not};

pub fn derive_is_variant(input: TokenStream) -> TokenStream {
    let DeriveInput::Enum(enum_def) = parse_derive_input(input) else {
        panic!("IsVariant can only be derived for enums");
    };

    let is_variant_functions = enum_def.variants.iter().map(|variant| {
        let func_name = Ident::new(&format!("is_{}", change_casing(&variant.ident.to_string(), Casing::Snake)));

        quote!(
            #[doc = concat!("Returns `true` if the enum is the variant `", stringify!(#variant_ident), "`.")]
            #[inline]
            pub const fn (@ func_name)(&self) -> bool {
                matches!(self, Self::(@ variant.ident.clone()){..})
            }
        )
    }).collect();

    impl_item(&enum_def, None, is_variant_functions)
}
