use forsith_shared::casing::{Casing, change_casing};
use proc_macro::{Ident, Span, TokenStream};
use crate::utils::{ItemType, impl_item, parse_enum_variants, parse_item, parse_struct_fields,};

pub fn derive_is_variant(input: TokenStream) -> TokenStream {
    let mut input = input.into_iter().peekable();

    let item = parse_item(&mut input);
    assert_eq!(item.ty(), &ItemType::Enum, "IsVariant can only be derived for enums, found {:?}", item.ty());

    let variants = parse_enum_variants(&mut input);

    let mut functions = TokenStream::new();
    for (variant_ident, _, _) in variants.into_iter() {
        let func_name = Ident::new(&format!("is_{}", change_casing(&variant_ident.to_string(), Casing::Snake)), Span::call_site());

        functions.extend(quote!(
            #[doc = concat!("Returns `true` if the enum is the variant `", stringify!(#variant_ident), "`.")]
            #[inline]
            pub fn (@ func_name)(self) -> bool {
                matches!(self, Self::(@ variant_ident))
            }
        ))
    }

    impl_item(&item, None, functions)
}


pub fn derive_deref_mut(input: TokenStream) -> TokenStream {
    let mut input = input.into_iter().peekable();

    let item = parse_item(&mut input);
    assert_eq!(item.ty(), &ItemType::Struct, "DerefMut can only be derived for structs, found {:?}", item.ty());

    let fields = parse_struct_fields(&mut input);

    let mut deref_fields = fields.iter().filter(|(_, _, attr)| attr.iter().any(|a| a.name().to_string() == "deref_mut"));
    assert!(deref_fields.clone().count() <= 1, "Deref can only be derived for structs with at most one field marked with #[deref_mut]");

    let deref_field = deref_fields.next().unwrap_or_else(|| {
        assert!(fields.len() == 1, "Deref can only be derived for structs with a single field if no field is marked with #[deref_mut]");
        &fields[0]
    });

    impl_item(&item, Some(quote!(std::ops::DerefMut)), quote!(
        fn deref_mut(&mut self) -> &mut Self::Target {
           &mut self.(@ deref_field.0.clone())
        }
    ))
}

pub fn derive_deref(input: TokenStream) -> TokenStream {
    let mut input = input.into_iter().peekable();

    let item = parse_item(&mut input);
    assert_eq!(item.ty(), &ItemType::Struct, "Deref can only be derived for structs, found {:?}", item.ty());

    let fields = parse_struct_fields(&mut input);

    let mut deref_fields = fields.iter().filter(|(_, _, attr)| attr.iter().any(|a| a.name().to_string() == "deref"));
    assert!(deref_fields.clone().count() <= 1, "Deref can only be derived for structs with at most one field marked with #[deref]");

    let deref_field = deref_fields.next().unwrap_or_else(|| {
        assert!(fields.len() == 1, "Deref can only be derived for structs with a single field if no field is marked with #[deref]");
        &fields[0]
    });

    impl_item(&item, Some(quote!(std::ops::Deref)), quote!(
        type Target = (@ deref_field.1.clone());

        fn deref(&self) -> &Self::Target {
           &self.(@ deref_field.0.clone())
        }
    ))
}
