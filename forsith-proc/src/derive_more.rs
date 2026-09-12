use forsith_base::{casing::{Casing, change_casing}, proc_macro::{Ident, TokenStream, items::{StructField, generating::impl_item, parsing::{DeriveInput, parse_derive_input}}}, quote};

pub fn derive_is_variant(input: TokenStream) -> TokenStream {
    let mut input = input.into_iter().peekable();

    let DeriveInput::Enum(enum_def) = parse_derive_input(&mut input) else {
        panic!("IsVariant can only be derived for enums");
    };

    let is_variant_functions = enum_def.variants.iter().map(|variant| {
        let func_name = Ident::new(&format!("is_{}", change_casing(&variant.ident.to_string(), Casing::Snake)));

        quote!(
            #[doc = concat!("Returns `true` if the enum is the variant `", stringify!(#variant_ident), "`.")]
            #[inline]
            pub fn (@ func_name)(self) -> bool {
                matches!(self, Self::(@ variant.ident.clone()))
            }
        )
    }).collect();

    impl_item(&enum_def, None, is_variant_functions)
}

pub fn choose_singular_struct_field<'a>(fields: &'a [StructField], attribute: &'static str) -> &'a StructField {
    let mut attributed_fields = fields.iter().filter(|field| field.attributes.iter().any(|a| a.name.to_string() == attribute));
    assert!(attributed_fields.clone().count() <= 1, "Only one field can be marked with #[{attribute}]");

    attributed_fields.next().unwrap_or_else(|| {
        assert!(fields.len() == 1, "Only one field can be present in the struct if no field is marked with #[{attribute}]");
        &fields[0]
    })
}


pub fn derive_deref_mut(input: TokenStream) -> TokenStream {
    let mut input = input.into_iter().peekable();

    let DeriveInput::Struct(struct_def) = parse_derive_input(&mut input) else {
        panic!("DerefMut can only be derived for structs");
    };

    let deref_field = choose_singular_struct_field(&struct_def.fields, "deref_mut");

    impl_item(&struct_def, Some(quote!(std::ops::DerefMut)), quote!(
        fn deref_mut(&mut self) -> &mut Self::Target {
           &mut self.(@ deref_field.name.clone())
        }
    ))
}

pub fn derive_deref(input: TokenStream) -> TokenStream {
    let mut input = input.into_iter().peekable();

    let DeriveInput::Struct(struct_def) = parse_derive_input(&mut input) else {
        panic!("Deref can only be derived for structs");
    };

    let deref_field = choose_singular_struct_field(&struct_def.fields, "deref");

    impl_item(&struct_def, Some(quote!(std::ops::Deref)), quote!(
        type Target = (@ deref_field.ty.clone());

        fn deref(&self) -> &Self::Target {
           &self.(@ deref_field.name.clone())
        }
    ))
}
