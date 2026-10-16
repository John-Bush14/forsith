//! SSFD: Singular Struct-Field Derive
//! This module contains the helpers and implementations for derive macros operating on a single
//! field of a struct.

use forsith_base::{
    proc_macro::{
        TokenStream,
        items::{
            generating::impl_item,
            parsing::{DeriveInput, parse_derive_input},
            struct_definition::{StructDefinition, StructField, StructFields},
        },
    },
    quote,
};

pub fn choose_singular_struct_field<'a>(
    fields: &'a StructFields,
    attribute: &'static str,
) -> StructField<'a> {
    let mut attributed_fields = fields.iter().filter(|field| {
        field
            .attributes
            .iter()
            .any(|a| a.name.to_string() == attribute)
    });

    let field = attributed_fields.next().unwrap_or_else(|| {
        assert!(
            fields.len() == 1,
            "Only one field can be present in the struct if no field is marked with #[{attribute}]"
        );
        fields.get(0).unwrap()
    });

    assert!(
        attributed_fields.next().is_none(),
        "Only one field can be marked with #[{attribute}]"
    );

    field
}

pub fn singular_struct_field_derive(
    input: TokenStream,
    attribute: &'static str,
    derive: fn(&StructDefinition, StructField) -> TokenStream,
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
                    &self.(@ deref_field.name)
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
                    &mut self.(@ deref_field.name)
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
                type Output = <(@ index_field.ty.clone()) as std::ops::Index<usize>>::Output;

                fn index(&self, index: usize) -> &Self::Output {
                    &self.(@ index_field.name)[index]
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
                    &mut self.(@ index_field.name)[index]
                }
            ),
        )
    })
}

pub fn derive_not(input: TokenStream) -> TokenStream {
    singular_struct_field_derive(input, "not", |struct_def, not_field| {
        impl_item(
            struct_def,
            Some(quote!(std::ops::Not)),
            quote!(
                type Output = Self;

                fn not(self) -> Self::Output {
                    Self {
                        (@ not_field.name.clone()): !self.(@ not_field.name),
                        ..self
                    }
                }
            ),
        )
    })
}
