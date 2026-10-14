use forsith_base::{
    casing::{Casing, change_casing},
    proc_macro::{
        Ident, Literal, TokenStream, TokenTree,
        items::{
            EnumDefinition, EnumVariant, ItemDefinition, StructDefinition, StructField,
            generating::impl_item,
            parsing::{DeriveInput, parse_derive_input},
        },
    },
    quote,
};

pub fn derive_is_variant(input: TokenStream) -> TokenStream {
    let DeriveInput::Enum(enum_def) = parse_derive_input(input) else {
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

pub fn item_repr(item: &impl ItemDefinition) -> Ident {
    item.attributes()
        .iter()
        .find(|attr| attr.name == Ident::new("repr"))
        .map_or_else(
            || Ident::new("usize"),
            |attr| {
                let args = attr
                    .args
                    .as_ref()
                    .expect("repr attribute must have arguments");

                match args.first() {
                    Some(TokenTree::Ident(ident)) if ident == &Ident::new("C") => {
                        panic!("repr(C) is not currently supported")
                    }
                    Some(TokenTree::Ident(ident)) => ident.clone(),
                    _ => panic!("repr attribute must have an identifier argument"),
                }
            },
        )
}

pub fn enum_integer_discriminants(
    enum_def: &EnumDefinition,
) -> (Vec<&EnumVariant>, Option<&EnumVariant>) {
    let mut wildcard_variant = None;

    (
        enum_def
            .variants
            .iter()
            .filter(|&variant| {
                let Some(discriminant) = &variant.discriminant else {
                    return false;
                };
                match discriminant.as_slice() {
                    [TokenTree::Literal(lit)] => {
                        assert!(
                            matches!(lit, Literal::Integer(_, _)),
                            "Discriminant must be a number literal"
                        );
                        true
                    }
                    [TokenTree::Ident(ident)] if ident == &Ident::new("_") => {
                        assert!(
                            wildcard_variant.is_none(),
                            "Only one wildcard variant is allowed"
                        );
                        wildcard_variant = Some(variant);
                        false
                    }
                    _ => panic!("Discriminant must be a number literal or a wildcard"),
                }
            })
            .collect(),
        wildcard_variant,
    )
}

pub fn assert_enum_unit_variants(enum_def: &EnumDefinition) {
    assert!(
        enum_def
            .variants
            .iter()
            .all(|variant| variant.fields.is_none()),
        "Can only be derived for enums with unit variants"
    );
}

pub fn generate_into_discriminant_match_arms(
    variants: &[&EnumVariant],
    result: bool
) -> TokenStream {
    variants
        .iter()
        .map(|variant| {
            let mut discriminant = variant.discriminant.clone().unwrap();
            if result {
                discriminant = quote!(Ok((@ discriminant)));
            }
            quote!(Self::(@ variant.ident.clone()) => (@ discriminant))
        })
        .collect::<TokenStream>()
}

pub fn generate_from_discriminant_match_arms(
    variants: &[&EnumVariant],
    result: bool
) -> TokenStream {
    variants
        .iter()
        .map(|variant| {
            let discriminant = variant.discriminant.clone().unwrap();
            let mut variant = quote!(Self::(@ variant.ident.clone()));
            if result {
                variant = quote!(Ok((@ variant)));
            }
            quote!((@ discriminant) => Self::(@ variant))
        })
        .collect::<TokenStream>()
}

pub fn derive_into_num(input: TokenStream) -> TokenStream {
    let DeriveInput::Enum(enum_def) = parse_derive_input(input) else {
        panic!("IntoEnum can only be derived for enums");
    };

    assert_enum_unit_variants(&enum_def);
    let repr = item_repr(&enum_def);
    let (variants, _) = enum_integer_discriminants(&enum_def);
    let match_arms = generate_into_discriminant_match_arms(&variants, false);

    assert!(
        variants.len() == enum_def.variants.len(),
        "IntoNum can only be derived for enums with all variants having a discriminant"
    );

    impl_item(
        &enum_def,
        Some(quote!(std::convert::Into<(@ repr.clone())>)),
        quote!(
            fn into(self) -> (@ repr) {
                match self {
                    (@ match_arms)
                }
            }
        ),
    )
}

pub fn derive_try_into_num(input: TokenStream) -> TokenStream {
    let DeriveInput::Enum(enum_def) = parse_derive_input(input) else {
        panic!("TryIntoNum can only be derived for enums");
    };

    assert_enum_unit_variants(&enum_def);
    let repr = item_repr(&enum_def);
    let (variants, _) = enum_integer_discriminants(&enum_def);
    let match_arms = generate_into_discriminant_match_arms(&variants, true);

    impl_item(
        &enum_def,
        Some(quote!(std::convert::TryInto<(@ repr.clone())>)),
        quote!(
            type Error = ();

            fn try_into(self) -> Result<(@ repr), Self::Error> {
                match self {
                    (@ match_arms)
                    _ => Err(()),
                }
            }
        ),
    )
}

pub fn derive_from_num(input: TokenStream) -> TokenStream {
    let DeriveInput::Enum(enum_def) = parse_derive_input(input) else {
        panic!("FromNum can only be derived for enums");
    };

    assert_enum_unit_variants(&enum_def);
    let repr = item_repr(&enum_def);
    let (variants, Some(wildcard)) = enum_integer_discriminants(&enum_def) else {
        panic!("FromNum can only be derived for enums with a wildcard variant to be infallibly matched, try using TryFromNum instead");
    };
    let match_arms = generate_from_discriminant_match_arms(&variants, false);

    impl_item(
        &enum_def,
        Some(quote!(std::convert::From<(@ repr.clone())>)),
        quote!(
            fn from(value: (@ repr)) -> Self {
                match value {
                    (@ match_arms)
                    _ => Self::(@ wildcard.ident.clone())
                }
            }
        ),
    )
}

pub fn derive_try_from_num(input: TokenStream) -> TokenStream {
    let DeriveInput::Enum(enum_def) = parse_derive_input(input) else {
        panic!("TryFromNum can only be derived for enums");
    };

    assert_enum_unit_variants(&enum_def);
    let repr = item_repr(&enum_def);
    let (variants, None) = enum_integer_discriminants(&enum_def) else {
        panic!("TryFromNum can only be derived for enums without a wildcard variant to be fallible, try using FromNum instead");
    };
    let match_arms = generate_from_discriminant_match_arms(&variants, true);

    impl_item(
        &enum_def,
        Some(quote!(std::convert::TryFrom<(@ repr.clone())>)),
        quote!(
            type Error = ();

            fn try_from(value: (@ repr)) -> Result<Self, Self::Error> {
                match value {
                    (@ match_arms)
                    _ => Err(()),
                }
            }
        ),
    )
}

pub fn choose_singular_struct_field<'a>(
    fields: &'a [StructField],
    attribute: &'static str,
) -> &'a StructField {
    let mut attributed_fields = fields.iter().filter(|field| {
        field
            .attributes
            .iter()
            .any(|a| a.name.to_string() == attribute)
    });

    assert!(
        attributed_fields.clone().count() <= 1,
        "Only one field can be marked with #[{attribute}]"
    );

    attributed_fields.next().unwrap_or_else(|| {
        assert!(
            fields.len() == 1,
            "Only one field can be present in the struct if no field is marked with #[{attribute}]"
        );
        &fields[0]
    })
}

pub fn singular_struct_field_derive(
    input: TokenStream,
    attribute: &'static str,
    derive: fn(&StructDefinition, &StructField) -> TokenStream,
) -> TokenStream {
    let DeriveInput::Struct(struct_def) = parse_derive_input(input) else {
        panic!("{attribute} can only be derived for structs");
    };

    let field = choose_singular_struct_field(&struct_def.fields, attribute);

    derive(&struct_def, field)
}

pub fn derive_deref(input: TokenStream) -> TokenStream {
    singular_struct_field_derive(input, "deref", |struct_def, deref_field| {
        impl_item(
            struct_def,
            Some(quote!(std::ops::Deref)),
            quote!(
                type Target = (@ deref_field.ty.clone());

                fn deref(&self) -> &Self::Target {
                    &self.(@ deref_field.name.clone())
                }
            ),
        )
    })
}

pub fn derive_deref_mut(input: TokenStream) -> TokenStream {
    singular_struct_field_derive(input, "deref_mut", |struct_def, deref_field| {
        impl_item(
            struct_def,
            Some(quote!(std::ops::DerefMut)),
            quote!(
                fn deref_mut(&mut self) -> &mut Self::Target {
                    &mut self.(@ deref_field.name.clone())
                }
            ),
        )
    })
}

pub fn derive_index(input: TokenStream) -> TokenStream {
    singular_struct_field_derive(input, "index", |struct_def, index_field| {
        impl_item(
            struct_def,
            Some(quote!(std::ops::Index<usize>)),
            quote!(
                type Output = (@ index_field.ty.clone());

                fn index(&self, index: usize) -> &Self::Output {
                    &self.(@ index_field.name.clone())[index]
                }
            ),
        )
    })
}

pub fn derive_index_mut(input: TokenStream) -> TokenStream {
    singular_struct_field_derive(input, "index_mut", |struct_def, index_field| {
        impl_item(
            struct_def,
            Some(quote!(std::ops::IndexMut<usize>)),
            quote!(
                fn index_mut(&mut self, index: usize) -> &mut Self::Output {
                    &mut self.(@ index_field.name.clone())[index]
                }
            ),
        )
    })
}
