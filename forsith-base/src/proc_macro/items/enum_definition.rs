use core::iter::Peekable;

use crate::proc_macro::{
    Delimiter, Group, Ident, PunctChar, TokenStream, TokenTree, items::{
        Attribute, GenericsDefinition, ItemDefinition, ItemPrefix, ItemType, Visibility, parsing::{parse_attributed_fields_generic, parse_type},
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
            ty: parse_type(stream),
        }
    }
}

impl EnumVariantNamedField {
    fn parse(
        stream: &mut Peekable<impl Iterator<Item = TokenTree>>,
        attributes: Vec<Attribute>,
    ) -> Self {
        let TokenTree::Ident(name) = stream.next().expect("Expected field name, found None") else {
            panic!("Expected field name, found non-ident token");
        };

        assert!(
            matches!(stream.next(), Some(TokenTree::Punct(ref punct)) if punct.char() == PunctChar::Colon),
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
        let name = match stream.next() {
            Some(TokenTree::Ident(name)) => name,
            tt => {
                panic!("Expected ident or attribute in enum variants, found `{tt:?}`");
            }
        };

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
                TokenTree::Punct(punct) if punct.char() == PunctChar::Equal => {
                    variant.discriminant = Some(parse_type(stream));
                    break; // parse_type consumes comma
                }
                TokenTree::Punct(punct) if punct.char() == PunctChar::Comma => {
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

        Self::assemble(prefix, Self::parse_variants(ts))
    }

    #[must_use]
    pub fn assemble(prefix: ItemPrefix, variants: Vec<EnumVariant>) -> Self {
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
            generics: prefix.generics,
            variants,
        }
    }

    pub fn parse_variants(input: &mut impl Iterator<Item = TokenTree>) -> Vec<EnumVariant> {
        let mut variants = Vec::new();
        let mut attributes = Vec::new();

        let group = match input.next() {
            Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace => group,
            t => panic!("Expected group of enum variants, found `{t:?}`"),
        };

        let mut stream = group.take_stream().into_iter().peekable();

        while let Some(tt) = stream.peek() {
            if let TokenTree::Punct(punct) = tt
                && punct.char() == PunctChar::Hash
            {
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
