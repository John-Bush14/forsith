use forsith_base::{
    proc_macro::{
        Ident, Literal, TokenStream, TokenTree,
        items::{
            ItemDefinition,
            enum_definition::{EnumDefinition, EnumVariant},
            generating::impl_item,
            parsing::{DeriveInput, parse_derive_input},
        },
    },
    quote,
};

pub fn enum_integer_discriminant_variants(enum_def: &EnumDefinition) -> Vec<&EnumVariant> {
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
                        "Discriminant must be a number literal but is {lit:?}",
                    );
                    true
                }
                _ => panic!("Discriminant must be a number literal"),
            }
        })
        .collect()
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
                    Some(tt) if tt.is_ident("C") => {
                        panic!("repr(C) is not currently supported")
                    }
                    Some(TokenTree::Ident(ident)) => ident.clone(),
                    _ => panic!("repr attribute must have an identifier argument"),
                }
            },
        )
}

pub fn assert_enum_unit_variants(enum_def: &EnumDefinition) {
    assert!(
        enum_def
            .variants
            .iter()
            .all(|variant| variant.fields.is_empty()),
        "Can only be derived for enums with unit variants"
    );
}

pub fn generate_into_discriminant_match_arms(
    variants: &[&EnumVariant],
    result: bool,
) -> TokenStream {
    variants
        .iter()
        .map(|variant| {
            let mut discriminant = variant.discriminant.clone().unwrap();
            if result {
                discriminant = quote!(Ok((@ discriminant)));
            }
            quote!((@ variant.ident.clone()) => (@ discriminant),)
        })
        .collect::<TokenStream>()
}

pub fn generate_from_discriminant_match_arms(
    variants: &[&EnumVariant],
    result: bool,
) -> TokenStream {
    variants
        .iter()
        .map(|variant| {
            let discriminant = variant.discriminant.clone().unwrap();
            let mut variant = quote!(Self::(@ variant.ident.clone()));
            if result {
                variant = quote!(Ok((@ variant)));
            }
            quote!((@ discriminant) => (@ variant),)
        })
        .collect::<TokenStream>()
}

pub fn derive_into_num(input: TokenStream) -> TokenStream {
    let DeriveInput::Enum(enum_def) = parse_derive_input(input) else {
        panic!("IntoEnum can only be derived for enums");
    };

    assert_enum_unit_variants(&enum_def);
    let repr = item_repr(&enum_def);
    let variants = enum_integer_discriminant_variants(&enum_def);
    let match_arms = generate_into_discriminant_match_arms(&variants, false);

    assert!(
        variants.len() == enum_def.variants.len(),
        "IntoNum can only be derived for enums with all variants having a discriminant"
    );

    let enum_type =
        quote!((@ enum_def.name.clone())<(@ enum_def.generics.inner_param_usage(None))>);

    quote! {
        impl(@ enum_def.generics.param_definition()) std::convert::From<(@ enum_type.clone())> for (@ repr) {
            fn from(value: (@ enum_type)) -> Self {
                use (@ enum_def.name)::*;
                match value {
                    (@ match_arms)
                }
            }
        }
    }
}

pub fn derive_try_into_num(input: TokenStream) -> TokenStream {
    let DeriveInput::Enum(enum_def) = parse_derive_input(input) else {
        panic!("TryIntoNum can only be derived for enums");
    };

    assert_enum_unit_variants(&enum_def);
    let repr = item_repr(&enum_def);
    let variants = enum_integer_discriminant_variants(&enum_def);
    let match_arms = generate_into_discriminant_match_arms(&variants, true);

    let enum_type =
        quote!((@ enum_def.name.clone())<(@ enum_def.generics.inner_param_usage(None))>);

    quote! {
        impl(@ enum_def.generics.param_definition()) std::convert::TryFrom<(@ enum_type.clone())> for (@ repr) {
            fn try_from(value: (@ enum_type)) -> Self {
                use (@ enum_def.name)::*;
                match value {
                    (@ match_arms)
                    _ => Err(()),
                }
            }
        }
    }
}

pub fn derive_from_num(input: TokenStream) -> TokenStream {
    let DeriveInput::Enum(enum_def) = parse_derive_input(input) else {
        panic!("FromNum can only be derived for enums");
    };

    assert_enum_unit_variants(&enum_def);
    let repr = item_repr(&enum_def);
    let variants = enum_integer_discriminant_variants(&enum_def);
    let match_arms = generate_from_discriminant_match_arms(&variants, false);

    let wildcard = enum_def
        .variants
        .iter()
        .find(|variant| variant.attributes.iter().any(|attr| attr.name == Ident::new("default")))
        .expect("FromNum can only be derived for enums with a wildcard variant, please use TryFromNum instead");

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
    let variants = enum_integer_discriminant_variants(&enum_def);
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
