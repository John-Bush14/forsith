use forsith_base::{casing::{Casing, change_casing}, proc_macro::{Ident, TokenStream, TokenTree}, quote};
use crate::utils::{Attribute, ItemContent, impl_item, parse_item};

pub fn derive_is_variant(input: TokenStream) -> TokenStream {
    let mut input = input.into_iter().peekable();

    let (item, ItemContent::Enum(variants)) = parse_item(&mut input) else {
        panic!("IsVariant can only be derived for enums");
    };

    let is_variant_functions = variants.into_iter().map(|(variant_ident, _, _)| {
        let func_name = Ident::new(&format!("is_{}", change_casing(&variant_ident.to_string(), Casing::Snake)));

        quote!(
            #[doc = concat!("Returns `true` if the enum is the variant `", stringify!(#variant_ident), "`.")]
            #[inline]
            pub fn (@ func_name)(self) -> bool {
                matches!(self, Self::(@ variant_ident))
            }
        )
    }).collect();

    impl_item(&item, None, is_variant_functions)
}

pub fn choose_singular_field<'a>(fields: &'a [(TokenTree, TokenStream, Vec<Attribute>)], attribute: &'static str) -> &'a (TokenTree, TokenStream, Vec<Attribute>) {
    let mut attributed_fields = fields.iter().filter(|(_, _, attr)| attr.iter().any(|a| a.name().to_string() == attribute));
    assert!(attributed_fields.clone().count() <= 1, "Deref can only be derived for structs with at most one field marked with #[deref_mut]");

    attributed_fields.next().unwrap_or_else(|| {
        assert!(fields.len() == 1, "Deref can only be derived for structs with a single field if no field is marked with #[deref_mut]");
        &fields[0]
    })
}


pub fn derive_deref_mut(input: TokenStream) -> TokenStream {
    let mut input = input.into_iter().peekable();

    let (item, ItemContent::Struct(fields)) = parse_item(&mut input) else {
        panic!("DerefMut can only be derived for structs");
    };

    let deref_field = choose_singular_field(&fields, "deref_mut");

    impl_item(&item, Some(quote!(std::ops::DerefMut)), quote!(
        fn deref_mut(&mut self) -> &mut Self::Target {
           &mut self.(@ deref_field.0.clone())
        }
    ))
}

pub fn derive_deref(input: TokenStream) -> TokenStream {
    let mut input = input.into_iter().peekable();

    let (item, ItemContent::Struct(fields)) = parse_item(&mut input) else {
        panic!("Deref can only be derived for structs");
    };

    let deref_field = choose_singular_field(&fields, "deref");

    impl_item(&item, Some(quote!(std::ops::Deref)), quote!(
        type Target = (@ deref_field.1.clone());

        fn deref(&self) -> &Self::Target {
           &self.(@ deref_field.0.clone())
        }
    ))
}
