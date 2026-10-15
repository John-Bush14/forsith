use crate::alloc::{string::ToString, vec::Vec};
use crate::proc_macro::items::enum_definition::EnumDefinition;
use crate::proc_macro::items::struct_definition::{StructDefinition, StructFields};
use crate::proc_macro::{
    Delimiter, Literal, PunctChar, TokenStream, TokenTree,
    items::{Attribute, GenericsDefinition, ItemPrefix, ItemType, Visibility},
};
use core::iter::{Peekable, once};

impl Attribute {
    pub fn parse_all(
        input: &mut Peekable<impl Iterator<Item = TokenTree>>,
        attributes: &mut Vec<Attribute>,
    ) {
        while let Some(TokenTree::Punct(punct)) = input.peek()
            && punct.char() == PunctChar::Hash
        {
            attributes.push(Self::parse(input));
        }
    }

    pub fn parse(input: &mut impl Iterator<Item = TokenTree>) -> Self {
        assert!(
            matches!(input.next(), Some(TokenTree::Punct(punct)) if punct.char() == PunctChar::Hash),
            "Tried to parse attribute, but first token was not `#`"
        );

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
            Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Parenthesis => {
                Some(group)
            }
            None => None,
            t => panic!("Expected (...) after `#ident` in attribute, found `{t:?}`"),
        };

        Self { name, args }
    }
}

impl Visibility {
    pub fn parse(input: &mut Peekable<impl Iterator<Item = TokenTree>>) -> Self {
        assert!(
            matches!(input.next(), Some(TokenTree::Ident(ident)) if ident.to_string() == "pub"),
            "Tried to parse visibility, but first token was not `pub`"
        );

        if let Some(TokenTree::Group(group)) = input.peek()
            && group.delimiter() == Delimiter::Parenthesis
        {
            let TokenTree::Group(group) = input.next().unwrap() else {
                unreachable!()
            };

            Self::SpecializedPublic(group.take_stream())
        } else {
            Self::Public
        }
    }

    pub fn parse_or_default(input: &mut Peekable<impl Iterator<Item = TokenTree>>) -> Self {
        if let Some(TokenTree::Ident(ident)) = input.peek()
            && ident.to_string() == "pub"
        {
            Self::parse(input)
        } else {
            Self::Private
        }
    }
}


pub fn parse_attributed_fields_generic<F, I, P>(stream: &mut Peekable<I>, parse: P) -> Vec<F>
where
    I: Iterator<Item = TokenTree>,
    P: Fn(&mut Peekable<I>, Vec<Attribute>) -> F,
{
    let mut attributes = Vec::new();
    let mut fields = Vec::new();

    while let Some(tt) = stream.peek() {
        if let TokenTree::Punct(punct) = tt
            && punct.char() == PunctChar::Hash
        {
            attributes.push(Attribute::parse(stream));
            continue;
        }

        fields.push(parse(stream, core::mem::take(&mut attributes)));
    }

    fields
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
                }
                TokenTree::Ident(ident) if ident.to_string() == "pub" => {
                    visibility = Visibility::parse(input);
                }
                TokenTree::Ident(ident) if ident.to_string() == "const" => constness = true,
                TokenTree::Ident(ident) if ident.to_string() == "unsafe" => unsafety = true,
                TokenTree::Ident(ident) if ident.to_string() == "async" => asyncness = true,
                TokenTree::Ident(_) => {
                    let TokenTree::Ident(item_type) = input.next().unwrap() else {
                        unreachable!()
                    };

                    let name = match input.next().expect("Expected item name after item type") {
                        TokenTree::Ident(name) => name,
                        tt => {
                            panic!("Expected item name after item type, found `{tt:?}`");
                        }
                    };
                    let generics = GenericsDefinition::parse(input);
                    let item_type = match item_type.to_string().as_str() {
                        "struct" => ItemType::Struct,
                        "enum" => ItemType::Enum,
                        _ => unimplemented!("Item type `{}` is not supported yet", item_type),
                    };

                    return Self {
                        attributes,
                        visibility,
                        constness,
                        unsafety,
                        asyncness,
                        name,
                        generics,
                        item_type,
                    };
                }
                t => panic!("Expected item type after attributes and modifiers, found `{t:?}`"),
            }
        }

        panic!("Expected item type after attributes and modifiers, found None");
    }
}

pub fn parse_type(input: &mut Peekable<impl Iterator<Item = TokenTree>>) -> TokenStream {
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

pub enum DeriveInput {
    Struct(StructDefinition),
    Enum(EnumDefinition),
}

#[must_use]
pub fn parse_derive_input(input: TokenStream) -> DeriveInput {
    let input = &mut input.into_iter().peekable();

    let item_prefix = ItemPrefix::parse(input);

    match item_prefix.item_type {
        ItemType::Struct => DeriveInput::Struct(StructDefinition::assemble(
            item_prefix,
            StructFields::parse(input),
        )),
        ItemType::Enum => DeriveInput::Enum(EnumDefinition::assemble(
            item_prefix,
            EnumDefinition::parse_variants(input),
        )),
        item_type => unimplemented!("Item type `{item_type:?}` is not supported yet"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::quote;

    #[test]
    fn parses_public_struct_with_generics() {
        let input = quote! { pub struct Example<T>(pub T, bool) };
        let mut iter = input.into_iter().peekable();
        let definition = StructDefinition::parse(&mut iter);

        assert_eq!(definition.name.to_string(), "Example");
        assert!(matches!(definition.visibility, Visibility::Public));
        assert_eq!(definition.fields.len(), 2);
        assert!(matches!(
            definition.fields.get(0).unwrap().visibility,
            Visibility::Public
        ));
        assert!(matches!(
            definition.fields.get(1).unwrap().visibility,
            Visibility::Private
        ));
    }

    #[test]
    fn parses_enum_variants_and_discriminants() {
        let input = quote! { enum Example { A(u8), B, C = u32 } };
        let mut iter = input.into_iter().peekable();
        let definition = EnumDefinition::parse(&mut iter);

        assert_eq!(definition.name.to_string(), "Example");
        assert_eq!(definition.variants.len(), 3);
        assert_eq!(definition.variants[0].fields.len(), 1);
        assert!(definition.variants[1].fields.is_empty());
        assert!(definition.variants[2].discriminant.is_some());
    }
}
