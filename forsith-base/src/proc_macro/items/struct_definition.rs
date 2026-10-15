use core::{
    iter::Peekable,
    ops::{Index, IndexMut},
};

use crate::proc_macro::{
    Delimiter, Ident, Literal, PunctChar, TokenStream, TokenTree,
    items::{
        Attribute, GenericsDefinition, ItemDefinition, ItemPrefix, ItemType, Visibility,
        parsing::parse_type,
    },
};

#[derive(Debug, Clone)]
pub struct NamedStructField {
    pub name: Ident,
    pub field: UnnamedStructField,
}

#[derive(Debug, Clone)]
pub struct UnnamedStructField {
    pub ty: TokenStream,
    pub visibility: Visibility,
    pub attributes: Vec<Attribute>,
}

#[derive(Debug, Clone)]
pub struct StructField<'a> {
    pub name: TokenTree,
    pub ty: &'a TokenStream,
    pub visibility: &'a Visibility,
    pub attributes: &'a Vec<Attribute>,
}

#[derive(Debug, Clone)]
pub enum StructFields {
    Named(Vec<NamedStructField>),
    Tuple(Vec<UnnamedStructField>),
    Unit,
}

impl StructFields {
    #[must_use]
    pub const fn len(&self) -> usize {
        match self {
            Self::Named(fields) => fields.len(),
            Self::Tuple(fields) => fields.len(),
            Self::Unit => 0,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = StructField<'_>> {
        (0..self.len()).map(move |i| self.get(i).expect("Should not happen"))
    }

    #[must_use]
    pub fn get(&self, index: usize) -> Option<StructField<'_>> {
        match self {
            Self::Named(fields) => fields.get(index).map(|f| StructField {
                name: TokenTree::Ident(f.name.clone()),
                ty: &f.field.ty,
                visibility: &f.field.visibility,
                attributes: &f.field.attributes,
            }),
            Self::Tuple(fields) => fields.get(index).map(|f| StructField {
                name: TokenTree::Literal(Literal::Integer(index, None)),
                ty: &f.ty,
                visibility: &f.visibility,
                attributes: &f.attributes,
            }),
            Self::Unit => None,
        }
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn parse_tuple(stream: &mut Peekable<impl Iterator<Item = TokenTree>>) -> Self {
        let mut fields = Vec::new();
        let mut attributes = Vec::new();

        while let Some(tt) = stream.peek() {
            if let TokenTree::Punct(punct) = tt
                && punct.char() == PunctChar::Hash
            {
                attributes.push(Attribute::parse(stream));
                continue;
            }

            fields.push(UnnamedStructField {
                visibility: Visibility::parse_or_default(stream),
                ty: parse_type(stream),
                attributes: core::mem::take(&mut attributes),
            });
        }

        Self::Tuple(fields)
    }

    pub fn parse_named(stream: &mut Peekable<impl Iterator<Item = TokenTree>>) -> Self {
        let mut attributes = Vec::new();
        let mut fields = Vec::new();

        while let Some(tt) = stream.peek() {
            if let TokenTree::Punct(punct) = tt
                && punct.char() == PunctChar::Hash
            {
                attributes.push(Attribute::parse(stream));
                continue;
            }

            let visibility = Visibility::parse_or_default(stream);

            let TokenTree::Ident(name) = stream.next().expect("Expected field name, found None")
            else {
                panic!("Expected field name, found non-ident token");
            };

            assert!(
                matches!(stream.next(), Some(TokenTree::Punct(ref punct)) if punct.char() == PunctChar::Colon),
                "Expected colon after field name`"
            );

            fields.push(NamedStructField {
                name,
                field: UnnamedStructField {
                    ty: parse_type(stream),
                    visibility,
                    attributes: core::mem::take(&mut attributes),
                },
            });
        }

        Self::Named(fields)
    }

    pub fn parse(stream: &mut impl Iterator<Item = TokenTree>) -> Self {
        let group = match stream.next() {
            Some(TokenTree::Group(group)) => group,
            Some(TokenTree::Punct(punct)) if punct.char() == PunctChar::Semicolon => {
                return Self::Unit;
            }
            t => panic!("Expected group of struct fields, found `{t:?}`"),
        };

        let (delimiter, stream) = group.decompose();
        let mut stream = stream.into_iter().peekable();

        match delimiter {
            Delimiter::Parenthesis => Self::parse_tuple(&mut stream),
            Delimiter::Brace => Self::parse_named(&mut stream),
            _ => panic!(
                "Expected struct fields to be in parentheses or braces, found `{delimiter:?}`"
            ),
        }
    }
}

pub struct StructDefinition {
    pub attributes: Vec<Attribute>,
    pub visibility: Visibility,
    pub name: Ident,
    pub generics: GenericsDefinition,
    pub fields: StructFields,
}

impl ItemDefinition for StructDefinition {
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

impl StructDefinition {
    pub fn parse(ts: &mut Peekable<impl Iterator<Item = TokenTree>>) -> Self {
        let prefix = ItemPrefix::parse(ts);

        Self::assemble(prefix, StructFields::parse(ts))
    }

    #[must_use]
    pub fn assemble(prefix: ItemPrefix, fields: StructFields) -> Self {
        assert!(
            matches!(prefix.item_type, ItemType::Struct),
            "Expected struct definition, found {:?}",
            prefix.item_type
        );
        assert!(!prefix.constness, "Structs cannot be const");
        assert!(!prefix.unsafety, "Structs cannot be unsafe");
        assert!(!prefix.asyncness, "Structs cannot be async");

        Self {
            attributes: prefix.attributes,
            visibility: prefix.visibility,
            name: prefix.name,
            generics: prefix.generics,
            fields,
        }
    }
}
