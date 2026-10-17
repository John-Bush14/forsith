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

mod convert;
pub use convert::{derive_from_num, derive_into_num, derive_try_from_num, derive_try_into_num};

mod ssfd;
pub use ssfd::{derive_deref, derive_deref_mut, derive_index, derive_index_mut, derive_not};

mod fmt;
pub use fmt::*;

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

#[allow(unreachable_patterns)]
pub fn derive_error(input: TokenStream) -> TokenStream {
    match parse_derive_input(input) {
        DeriveInput::Enum(enum_def) => {
            quote! {
                (@ impl_error(&enum_def))
                (@ handle_from_attributes_enum(&enum_def))
            }
        }
        DeriveInput::Struct(struct_def) => impl_error(&struct_def),
        _ => unimplemented!("Error can only be derived for enums and structs"),
    }
}

pub fn handle_from_attributes_enum(enum_def: &EnumDefinition) -> TokenStream {
    enum_def
        .variants
        .iter()
        .filter_map(|variant| {
            let _attr = variant.fields.get_unnamed(0).and_then(|field| {
                field
                    .attributes
                    .iter()
                    .find(|attr| attr.name == Ident::new("from"))
            })?;

            assert!(
                variant.fields.len() == 1,
                "Only single-field variants can be marked with #[from]"
            );
            let field = &variant.fields.get_unnamed(0).unwrap();
            let ty = &field.ty;

            Some(quote! {
                (@ impl_item(enum_def, Some(quote!(core::convert::From<(@ ty.clone())>)), quote!(
                    fn from(value: (@ ty.clone())) -> Self {
                        Self::(@ variant.ident.clone())(value)
                    }
                )))
            })
        })
        .collect()
}

pub fn impl_error(def: &impl ItemDefinition) -> TokenStream {
    impl_item(def, Some(quote!(core::error::Error)), TokenStream::new())
}
