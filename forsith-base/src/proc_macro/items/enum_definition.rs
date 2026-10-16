use core::iter::Peekable;

use crate::proc_macro::{
    Delimiter, Group, Ident, PunctChar, TokenStream, TokenTree,
    items::{
        Attribute, GenericsDefinition, ItemDefinition, ItemPrefix, ItemType, Visibility,
        generics_definition::WhereClause,
        parsing::{parse_attributed_fields_generic, parse_type},
    },
};

pub struct EnumVariant {
    pub ident: Ident,
    pub fields: EnumVariantFields,
    pub attributes: Vec<Attribute>,
    pub discriminant: Option<TokenStream>,
}

pub enum EnumVariantFields {
    Unit,
    Tuple(Vec<EnumVariantUnnamedField>),
    Struct(Vec<EnumVariantNamedField>),
}

pub struct EnumVariantUnnamedField {
    pub attributes: Vec<Attribute>,
    pub ty: TokenStream,
}

pub struct EnumVariantNamedField {
    pub name: Ident,
    pub field: EnumVariantUnnamedField,
}

pub struct EnumDefinition {
    pub name: Ident,
    pub generics: GenericsDefinition,
    pub attributes: Vec<Attribute>,
    pub visibility: Visibility,
    pub variants: Vec<EnumVariant>,
}

impl ItemDefinition for EnumDefinition {
    fn name(&self) -> &Ident {
        &self.name
    }
    fn generics(&self) -> &GenericsDefinition {
        &self.generics
    }
    fn attributes(&self) -> &[Attribute] {
        &self.attributes
    }
}

impl EnumVariantFields {
    #[must_use]
    pub const fn len(&self) -> usize {
        match self {
            Self::Unit => 0,
            Self::Tuple(fields) => fields.len(),
            Self::Struct(fields) => fields.len(),
        }
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn parse(group: Group) -> Self {
        let (delimiter, stream) = group.decompose();
        let mut stream = stream.into_iter().peekable();

        match delimiter {
            Delimiter::Parenthesis => Self::Tuple(parse_attributed_fields_generic(
                &mut stream,
                EnumVariantUnnamedField::parse,
            )),
            Delimiter::Brace => Self::Struct(parse_attributed_fields_generic(
                &mut stream,
                EnumVariantNamedField::parse,
            )),
            _ => panic!(
                "Expected enum variant fields to be in parentheses or braces, found `{delimiter:?}`"
            ),
        }
    }
}

impl EnumVariantUnnamedField {
    fn parse(
        stream: &mut Peekable<impl Iterator<Item = TokenTree>>,
        attributes: Vec<Attribute>,
    ) -> Self {
        Self {
            attributes,
            ty: parse_type::<true, true, false, false, false>(stream),
        }
    }
}

impl EnumVariantNamedField {
    fn parse(
        stream: &mut Peekable<impl Iterator<Item = TokenTree>>,
        attributes: Vec<Attribute>,
    ) -> Self {
        let name = stream
            .next()
            .expect("Expected field name, found None")
            .ident()
            .expect("Expected identifier as field name");

        assert!(
            stream
                .next()
                .is_some_and(|tt| tt.is_punct(PunctChar::Colon)),
            "Expected colon after field name`"
        );

        Self {
            name,
            field: EnumVariantUnnamedField::parse(stream, attributes),
        }
    }
}

impl EnumVariant {
    fn parse(
        stream: &mut Peekable<impl Iterator<Item = TokenTree>>,
        mut attributes: Vec<Attribute>,
    ) -> Self {
        let name = stream
            .next()
            .expect("Expected enum variant name, found None")
            .ident()
            .expect("Expected identifier as enum variant name");

        let mut variant = Self {
            ident: name,
            fields: EnumVariantFields::Unit,
            attributes: core::mem::take(&mut attributes),
            discriminant: None,
        };

        while let Some(tt) = stream.next() {
            match tt {
                TokenTree::Group(group) => {
                    variant.fields = EnumVariantFields::parse(group);
                }
                tt if tt.is_punct(PunctChar::Equal) => {
                    variant.discriminant =
                        Some(parse_type::<true, true, false, false, false>(stream));
                    break; // parse_type consumes comma
                }
                tt if tt.is_punct(PunctChar::Comma) => {
                    break;
                }
                _ => panic!(
                    "Expected `(`, `{{`, `=`, `,` or end of enum variants after variant name, found `{tt:?}`"
                ),
            }
        }

        variant
    }
}

impl EnumDefinition {
    pub fn parse(ts: &mut Peekable<impl Iterator<Item = TokenTree>>) -> Self {
        let prefix = ItemPrefix::parse(ts);

        Self::assemble(
            prefix,
            GenericsDefinition::parse_where_clauses(ts),
            Self::parse_variants(ts),
        )
    }

    #[must_use]
    pub fn assemble(
        prefix: ItemPrefix,
        where_clauses: Vec<WhereClause>,
        variants: Vec<EnumVariant>,
    ) -> Self {
        assert!(
            matches!(prefix.item_type, ItemType::Enum),
            "Expected enum definition, found {:?}",
            prefix.item_type
        );
        assert!(!prefix.constness, "Enums cannot be const");
        assert!(!prefix.unsafety, "Enums cannot be unsafe");
        assert!(!prefix.asyncness, "Enums cannot be async");

        Self {
            attributes: prefix.attributes,
            visibility: prefix.visibility,
            name: prefix.name,
            generics: GenericsDefinition::assemble(prefix.generic_params, where_clauses),
            variants,
        }
    }

    pub fn parse_variants(input: &mut impl Iterator<Item = TokenTree>) -> Vec<EnumVariant> {
        let mut variants = Vec::new();
        let mut attributes = Vec::new();

        let group = input
            .next()
            .expect("Expected enum variants group, found None")
            .delimited_group(Delimiter::Brace)
            .expect("Expected enum variants to be in braces");

        let mut stream = group.take_stream().into_iter().peekable();

        while let Some(tt) = stream.peek() {
            if tt.is_punct(PunctChar::Hash) {
                attributes.push(Attribute::parse(&mut stream));
                continue;
            }

            variants.push(EnumVariant::parse(
                &mut stream,
                core::mem::take(&mut attributes),
            ));
        }

        variants
    }
}
