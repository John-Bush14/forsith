use core::iter::Peekable;

use crate::proc_macro::{Delimiter, Group, Ident, PunctChar, TokenStream, TokenTree, items::{Attribute, GenericsDefinition, ItemDefinition, ItemPrefix, ItemType, Visibility, parsing::parse_type}};

pub struct EnumVariant {
    pub ident: Ident,
    pub fields: Option<Group>,
    pub attributes: Vec<Attribute>,
    pub discriminant: Option<TokenStream>,
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

        let mut iter = group.take_stream().into_iter().peekable();

        while let Some(token) = iter.peek() {
            match token {
                TokenTree::Ident(_) => {
                    let TokenTree::Ident(name) = iter.next().unwrap() else {
                        unreachable!()
                    };

                    let mut variant = EnumVariant {
                        ident: name,
                        fields: None,
                        attributes: core::mem::take(&mut attributes),
                        discriminant: None,
                    };

                    match iter.peek() {
                        Some(TokenTree::Group(group))
                            if group.delimiter() == Delimiter::Parenthesis =>
                        {
                            let TokenTree::Group(group) = iter.next().unwrap() else {
                                unreachable!()
                            };
                            variant.fields = Some(group);
                            if let Some(TokenTree::Punct(punct)) = iter.peek()
                                && punct.char() == PunctChar::Comma
                            {
                                let _ = iter.next();
                            }
                        }
                        Some(TokenTree::Punct(punct)) if punct.char() == PunctChar::Equal => {
                            let _ = iter.next();
                            variant.discriminant = Some(parse_type(&mut iter));
                        }
                        Some(TokenTree::Punct(punct)) if punct.char() == PunctChar::Comma => {
                            let _ = iter.next();
                        }
                        None => {}
                        token => panic!(
                            "Expected `(`, `=`, `,` or end of enum variants after variant name, found `{token:?}`"
                        ),
                    }

                    variants.push(variant);
                }
                TokenTree::Punct(punct) if punct.char() == PunctChar::Hash => {
                    attributes.push(Attribute::parse(&mut iter));
                }
                t => panic!("Expected ident or attribute in enum variants, found `{t:?}`"),
            }
        }

        variants
    }
}
