use crate::proc_macro::{Delimiter, Literal, PunctChar, TokenStream, TokenTree, items::{Attribute, EnumDefinition, EnumVariant, GenericDefinition, GenericsDefinition, ItemPrefix, ItemType, StructDefinition, StructField, Visibility}};
use std::{iter::{Peekable, once}};

impl Attribute {
    fn parse(input: &mut impl Iterator<Item = TokenTree>) -> Self {
        assert!(matches!(input.next(), Some(TokenTree::Punct(punct)) if punct.char() == PunctChar::Hash), "Tried to parse attribute, but first token was not `#`");

        let att = match input.next() {
            Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Bracket => group,
            t => panic!("Expected [...] after `#` in enum variants, found `{t:?}`"),
        };

        let mut att = att.take_stream().into_iter();

        let name = match att.next() {
            Some(TokenTree::Ident(ident)) => ident,
            t => panic!("Expected ident after `#` in attribute, found `{t:?}`"),
        };

        let args = match att.next() {
            Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Parenthesis => Some(group),
            None => None,
            t => panic!("Expected (...) after `#ident` in attribute, found `{t:?}`"),
        };

        Self { name, args }
    }
}

impl Visibility {
    fn parse(input: &mut Peekable<impl Iterator<Item = TokenTree>>) -> Self {
        assert!(matches!(input.next(), Some(TokenTree::Ident(ident)) if ident.to_string() == "pub"), "Tried to parse visibility, but first token was not `pub`");

        if let Some(TokenTree::Group(group)) = input.peek() && group.delimiter() == Delimiter::Parenthesis {
            let TokenTree::Group(group) = input.next().unwrap() else {unreachable!()};
            Self::SpecializedPublic(group.take_stream())
        } else {
            Self::Public
        }
    }

    fn parse_or_default(input: &mut Peekable<impl Iterator<Item = TokenTree>>) -> Self {
        if let Some(TokenTree::Ident(ident)) = input.peek() && ident.to_string() == "pub" {
            Self::parse(input)
        } else {
            Self::Private
        }
    }
}

impl ItemPrefix {
    pub fn parse(input: &mut Peekable<impl Iterator<Item = TokenTree>>) -> Self {
        let mut attributes = Vec::new();
        let mut visibility = Visibility::Private;
        let mut constness = false;
        let mut unsafety = false;
        let mut asyncness = false;

        while let Some(token) = input.peek() {
            match token {
                TokenTree::Punct(punct) if punct.char() == PunctChar::Hash => {
                    attributes.push(Attribute::parse(input));
                },
                TokenTree::Ident(ident) if ident.to_string() == "pub" => {
                    visibility = Visibility::parse(input);
                },
                TokenTree::Ident(ident) if ident.to_string() == "const" => constness = true,
                TokenTree::Ident(ident) if ident.to_string() == "unsafe" => unsafety = true,
                TokenTree::Ident(ident) if ident.to_string() == "async" => asyncness = true,
                TokenTree::Ident(_) => {
                    let TokenTree::Ident(item_type) = input.next().unwrap() else {unreachable!()};
                    let TokenTree::Ident(name) = input.next().expect("Expected item name after item type") else {unreachable!()};
                    let generics = GenericsDefinition::parse(input);
                    let item_type = match item_type.to_string().as_str() {
                        "struct" => ItemType::Struct,
                        "enum" => ItemType::Enum,
                        _ => unimplemented!("Item type `{}` is not supported yet", item_type),
                    };

                    return Self { attributes, visibility, constness, unsafety, asyncness, name, generics, item_type };
                },
                t => panic!("Expected item type after attributes and modifiers, found `{t:?}`"),
            }
        }

        panic!("Expected item type after attributes and modifiers, found None");
    }
}

fn parse_type(input: &mut Peekable<impl Iterator<Item = TokenTree>>) -> TokenStream {
    let mut ty_tokens = TokenStream::new();
    let mut nested = 0;

    for item in input {
        if let TokenTree::Punct(ref punct) = item {
            match punct.char() {
                PunctChar::LessThan => nested += 1,
                PunctChar::GreaterThan => nested -= 1,
                PunctChar::Comma if nested == 0 => break,
                _ => {}
            }
        }
        ty_tokens.extend(once(item));
    }

    ty_tokens
}

impl StructDefinition {
    pub fn parse(ts: &mut Peekable<impl Iterator<Item = TokenTree>>) -> Self {
        let prefix = ItemPrefix::parse(ts);

        Self::assemble(prefix, Self::parse_fields(ts))
    }

    #[must_use]
    pub fn assemble(prefix: ItemPrefix, fields: Vec<StructField>) -> Self {
        assert!(matches!(prefix.item_type, ItemType::Struct), "Expected struct definition, found {:?}", prefix.item_type);
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

    pub fn parse_fields(input: &mut impl Iterator<Item = TokenTree>) -> Vec<StructField> {
        let mut fields = Vec::new();

        let group = match input.next() {
            Some(TokenTree::Group(group)) => group,
            Some(TokenTree::Punct(punct)) if punct.char() == PunctChar::Semicolon => return fields,
            t => panic!("Expected group of struct fields, found `{t:?}`"),
        };

        let mut attributes = Vec::new();

        let (delimiter, stream) = group.decompose();
        let mut iter = stream.into_iter().peekable();

        match delimiter {
            Delimiter::Parenthesis => {
                while let Some(item) = iter.peek() {
                    match item {
                        TokenTree::Punct(punct) if punct.char() == PunctChar::Hash => {
                            attributes.push(Attribute::parse(&mut iter));
                        },
                        _ => {
                            let visibility = Visibility::parse_or_default(&mut iter);

                            let i = Literal::Integer(fields.len(), None);
                            fields.push(StructField {
                                name: TokenTree::Literal(i),
                                visibility,
                                ty: parse_type(&mut iter),
                                attributes: std::mem::take(&mut attributes)
                            });
                        }
                    }
                }

                return fields;

            },
            Delimiter::Brace => {
                while let Some(token) = iter.peek() {
                    match token {
                        TokenTree::Ident(_) => {
                            let visibility = Visibility::parse_or_default(&mut iter);

                            let TokenTree::Ident(name) = iter.next().unwrap() else {unreachable!()};

                            let colon = iter.next().expect("Expected colon after field name");
                            assert!(matches!(colon, TokenTree::Punct(ref punct) if punct.char() == PunctChar::Colon), "Expected colon after field name, found `{colon:?}`");

                            fields.push(StructField {
                                name: TokenTree::Ident(name),
                                ty: parse_type(&mut iter),
                                visibility,
                                attributes: std::mem::take(&mut attributes)
                            });
                        },
                        TokenTree::Punct(punct) if punct.char() == PunctChar::Hash => {
                            attributes.push(Attribute::parse(&mut iter));
                        },
                        t => panic!("Expected ident or attribute in struct fields, found `{t:?}`"),
                    }
                }
            },
            _ => panic!("Expected struct fields to be in parentheses or braces, found `{delimiter:?}`"),
        }

        fields
    }
}

impl EnumDefinition {
    pub fn parse(ts: &mut Peekable<impl Iterator<Item = TokenTree>>) -> Self {
        let prefix = ItemPrefix::parse(ts);

        Self::assemble(prefix, Self::parse_variants(ts))
    }

    #[must_use]
    pub fn assemble(prefix: ItemPrefix, variants: Vec<EnumVariant>) -> Self {
        assert!(matches!(prefix.item_type, ItemType::Enum), "Expected enum definition, found {:?}", prefix.item_type);
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
                    let TokenTree::Ident(name) = iter.next().unwrap() else {unreachable!()};

                    let mut variant = EnumVariant {
                        ident: name,
                        fields: None,
                        attributes: std::mem::take(&mut attributes),
                        discriminant: None,
                    };

                    match iter.peek() {
                        Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Parenthesis => {
                            let TokenTree::Group(group) = iter.next().unwrap() else {unreachable!()};
                            variant.fields = Some(group);
                            if let Some(TokenTree::Punct(punct)) = iter.peek() && punct.char() == PunctChar::Comma {
                                let _ = iter.next();
                            }
                        },
                        Some(TokenTree::Punct(punct)) if punct.char() == PunctChar::Equal => {
                            let _ = iter.next();
                            variant.discriminant = Some(parse_type(&mut iter));
                        },
                        Some(TokenTree::Punct(punct)) if punct.char() == PunctChar::Comma => {
                            let _ = iter.next();
                        },
                        None => {},
                        token => panic!("Expected `(`, `=`, `,` or end of enum variants after variant name, found `{token:?}`"),
                    }

                    variants.push(variant);
                },
                TokenTree::Punct(punct) if punct.char() == PunctChar::Hash => {
                    attributes.push(Attribute::parse(&mut iter));
                },
                t => panic!("Expected ident or attribute in enum variants, found `{t:?}`"),
            }
        }

        variants
    }
}

impl GenericsDefinition {
    fn parse(input: &mut Peekable<impl Iterator<Item = TokenTree>>) -> Self {
        let mut generics: Vec<GenericDefinition> = Vec::new();
        if let Some(TokenTree::Punct(punct)) = input.peek() && punct.char() == PunctChar::LessThan {
            let _ = input.next();

            loop {
                match input.next() {
                    Some(TokenTree::Ident(ident)) => {
                        let mut constraints = TokenStream::new();
                        if let Some(TokenTree::Punct(punct)) = input.peek() && punct.char() == PunctChar::Colon {
                            let _ = input.next();

                            let mut nested = 0;
                            while let Some(item) = input.peek() {
                                if let TokenTree::Punct(punct) = item {
                                    match punct.char() {
                                        PunctChar::LessThan => nested += 1,
                                        PunctChar::GreaterThan => {
                                            if nested == 0 {break;}
                                            nested -= 1;
                                        },
                                        PunctChar::Comma if nested == 0 => {let _ = input.next(); break},
                                        _ => {}
                                    }
                                }
                                constraints.extend(once(input.next().unwrap()));
                            }
                        }
                        generics.push(GenericDefinition::Type(ident, constraints));
                    },
                    Some(TokenTree::Punct(punct)) if punct.char() == PunctChar::GreaterThan => break,
                    Some(TokenTree::Punct(punct)) if punct.char() == PunctChar::Qoute => {
                        let lifetime_ident = match input.next() {
                            Some(TokenTree::Ident(ident)) => ident,
                            t => panic!("Expected lifetime name after `'`, found `{t:?}`"),
                        };
                        generics.push(GenericDefinition::Lifetime(lifetime_ident));

                        if let Some(TokenTree::Punct(punct)) = input.peek() && punct.char() == PunctChar::Comma {
                            let _ = input.next();
                        }
                    },
                    None => panic!("Expected Some after `<` in generics, found None"),
                    tt => panic!("Expected ident or `>` or \"'\" in generics, found `{tt:?}`"),
                }
            }
        }

        Self(generics)
    }
}

pub enum DeriveInput {
    Struct(StructDefinition),
    Enum(EnumDefinition),
}

pub fn parse_derive_input(input: &mut Peekable<impl Iterator<Item = TokenTree>>) -> DeriveInput {
    let item_prefix = ItemPrefix::parse(input);

    match item_prefix.item_type {
        ItemType::Struct => DeriveInput::Struct(StructDefinition::assemble(item_prefix, StructDefinition::parse_fields(input))),
        ItemType::Enum => DeriveInput::Enum(EnumDefinition::assemble(item_prefix, EnumDefinition::parse_variants(input))),
        item_type => unimplemented!("Item type `{item_type:?}` is not supported yet"),
    }
}

