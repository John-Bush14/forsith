use forsith_base::{
    casing::{Casing, change_casing},
    proc_macro::{
        Ident, TokenStream,
        items::{
            ItemDefinition,
            enum_definition::EnumDefinition,
            generating::impl_item,
            parsing::{DeriveInput, parse_derive_input},
        },
    },
    quote,
};

macro_rules! fmt_traits {
    ($($trait:ident),*) => {
        $(
            #[allow(non_snake_case)]
            pub fn $trait(input: TokenStream) -> TokenStream {
                derive_fmt_trait(input, Ident::new(stringify!($trait)))
            }
        )*
    };
}

fmt_traits!(
    Display, Octal, LowerHex, UpperHex, Binary, Pointer, LowerExp, UpperExp
);

fn derive_fmt_trait(input: TokenStream, r#trait: Ident) -> TokenStream {
    match parse_derive_input(input) {
        DeriveInput::Enum(def) => derive_fmt_trait_item(&def, r#trait, derive_fmt_body_enum),
        _ => unimplemented!("{trait} is currently only implemented for enums"),
    }
}

fn derive_fmt_trait_item<I: ItemDefinition>(
    def: &I,
    r#trait: Ident,
    body: fn(&I, &Ident) -> TokenStream,
) -> TokenStream {
    let attribute = Ident::from(change_casing(&r#trait, Casing::Snake));

    impl_item(
        def,
        Some(quote!(core::fmt::(@ r#trait))),
        quote! {
            fn fmt(&self, f: &mut core::fmt::Formatter< '_ >) -> core::fmt::Result {
                (@ body(def, &attribute))
            }
        },
    )
}

fn derive_fmt_body_enum(enum_def: &EnumDefinition, attribute: &Ident) -> TokenStream {
    let global = enum_def
        .attributes()
        .iter()
        .find(|attr| &attr.name == attribute)
        .map_or_else(
            || quote!("{_variant}"),
            |attr| {
                attr.args
                    .clone()
                    .expect("fmt attribute must have arguments")
                    .take_stream()
            },
        );

    let variant_cases = enum_def.variants.iter().map(|variant| {
        let variant_ident = &variant.ident;

        let attr = variant
            .attributes
            .iter()
            .find(|attr| &attr.name == attribute)
            .map_or_else(
                || todo!(),
                |attr| {
                    attr.args
                        .clone()
                        .expect("fmt attribute must have arguments")
                        .take_stream()
                },
            );

        quote! {
            Self::(@ variant_ident.clone()) (@ variant.unpack_pattern()) => format!((@ attr)),
        }
    }).collect::<TokenStream>();

    quote! {
        let _variant = match self {
            (@ variant_cases)
        };

        write!(f, (@ global))
    }
}
