use forsith_shared::casing::{Casing, change_casing};
use proc_macro::{Ident, Span, TokenStream};
use crate::utils::{ItemType, impl_item, parse_enum_variants, parse_item,};

pub fn derive_is_variant(input: TokenStream) -> TokenStream {
    let mut input = input.into_iter().peekable();

    let item = parse_item(&mut input);
    assert_eq!(item.ty(), &ItemType::Enum, "IsVariant can only be derived for enums, found {:?}", item.ty());

    let variants = parse_enum_variants(&mut input);

    let mut functions = TokenStream::new();
    for (variant_ident, _) in variants.into_iter() {
        let func_name = Ident::new(&format!("is_{}", change_casing(&variant_ident.to_string(), Casing::Snake)), Span::call_site());

        functions.extend(quote!(
            #[doc = concat!("Returns `true` if the enum is the variant `", stringify!(#variant_ident), "`.")]
            #[inline]
            pub fn (@ func_name)(self) -> bool {
                matches!(self, Self::(@ variant_ident))
            }
        ))
    }

    impl_item(&item, functions)
}
