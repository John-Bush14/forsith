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
        stream: &mut Peekable<impl Iterator<Item = TokenTree>>,
        attributes: &mut Vec<Self>,
    ) {
        while stream.peek().is_some_and(|tt| tt.is_punct(PunctChar::Hash)) {
            attributes.push(Self::parse(stream));
        }
    }

    pub fn parse(stream: &mut impl Iterator<Item = TokenTree>) -> Self {
        assert!(
            matches!(stream.next(), Some(TokenTree::Punct(punct)) if punct.char() == PunctChar::Hash),
            "Tried to parse attribute, but first token was not `#`"
        );

        let mut att_ts = stream
            .next()
            .expect("Expected attribute after `#`")
            .group()
            .expect("Expected group after `#`")
            .take_stream()
            .into_iter();

        let name = att_ts
            .next()
            .expect("Expected attribute name after `#`")
            .ident()
            .expect("Expected identifier as attribute name");

        let args = att_ts.next().map(|tt| {
            tt.delimited_group(Delimiter::Parenthesis)
                .expect("Expected (...) after attribute name")
        });

        Self { name, args }
    }
}

impl Visibility {
    pub fn parse(stream: &mut Peekable<impl Iterator<Item = TokenTree>>) -> Self {
        assert!(
            matches!(stream.next(), Some(TokenTree::Ident(ident)) if ident.to_string() == "pub"),
            "Tried to parse visibility, but first token was not `pub`"
        );

        if stream
            .peek()
            .is_some_and(|tt| tt.is_group(Delimiter::Parenthesis))
        {
            let group = stream.next().unwrap().group().unwrap();

            Self::SpecializedPublic(group.take_stream())
        } else {
            Self::Public
        }
    }

    pub fn parse_or_default(input: &mut Peekable<impl Iterator<Item = TokenTree>>) -> Self {
        if input.peek().is_some_and(|tt| tt.is_ident("pub")) {
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
        if tt.is_punct(PunctChar::Hash) {
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
                tt if tt.is_punct(PunctChar::Hash) => {
                    attributes.push(Attribute::parse(input));
                }
                tt if tt.is_ident("pub") => {
                    visibility = Visibility::parse(input);
                }
                tt if tt.is_ident("const") => constness = true,
                tt if tt.is_ident("unsafe") => unsafety = true,
                tt if tt.is_ident("async") => asyncness = true,
                TokenTree::Ident(_) => {
                    let item_type = input.next().unwrap().ident().unwrap();

                    let item_type = match item_type.to_string().as_str() {
                        "struct" => ItemType::Struct,
                        "enum" => ItemType::Enum,
                        _ => unimplemented!("Item type `{}` is not supported yet", item_type),
                    };

                    let name = input
                        .next()
                        .expect("Expected item name after item type")
                        .ident()
                        .expect("Expected identifier as item name");

                    let generic_params = GenericsDefinition::parse_params(input);

                    return Self {
                        attributes,
                        visibility,
                        constness,
                        unsafety,
                        asyncness,
                        name,
                        generic_params,
                        item_type,
                    };
                }
                tt => panic!("Expected item type after attributes and modifiers, found `{tt:?}`"),
            }
        }

        panic!("Expected item type after attributes and modifiers, found None");
    }
}

pub fn parse_type<
    const CONSUME: bool,
    const COMMA: bool,
    const COLON: bool,
    const WHERE_CLAUSE_RHS: bool,
    const LESSTHEN: bool,
>(
    input: &mut Peekable<impl Iterator<Item = TokenTree>>,
) -> TokenStream {
    assert!(
        COMMA || COLON,
        "parse_type must be called with at least one of COMMA or COLON set to true"
    );

    let mut ty_tokens = TokenStream::new();
    let mut nested = 0u32;

    while let Some(tt) = input.peek() {
        if let TokenTree::Punct(punct) = tt {
            match punct.char() {
                PunctChar::LessThan => nested += 1,
                PunctChar::GreaterThan if nested == 0 => {
                    if LESSTHEN {
                        break;
                    }
                    panic!("Unexpected `>` in type")
                }
                PunctChar::GreaterThan => nested -= 1,
                PunctChar::Comma if COMMA && nested == 0 => break,
                PunctChar::Semicolon | PunctChar::Equal if WHERE_CLAUSE_RHS && nested == 0 => break,
                PunctChar::Colon
                    if COLON
                        && nested == 0
                        && !punct.joint()
                        && !matches!(ty_tokens.last(), Some(tt) if tt.is_joint_punct()) =>
                {
                    break;
                }
                _ => {}
            }
        }

        if WHERE_CLAUSE_RHS && (tt.is_group(Delimiter::Brace) || tt.is_group(Delimiter::Parenthesis))
        {
            break;
        }

        ty_tokens.extend(once(input.next().unwrap()));
    }

    if CONSUME {
        let _ = input.next();
    }

    ty_tokens
}

pub enum DeriveInput {
    Struct(StructDefinition),
    Enum(EnumDefinition),
}

#[must_use]
pub fn parse_derive_input(input: TokenStream) -> DeriveInput {
    let stream = &mut input.into_iter().peekable();

    let item_prefix = ItemPrefix::parse(stream);

    match item_prefix.item_type {
        ItemType::Struct => DeriveInput::Struct(StructDefinition::assemble(
            item_prefix,
            GenericsDefinition::parse_where_clauses(stream),
            StructFields::parse(stream),
        )),
        ItemType::Enum => DeriveInput::Enum(EnumDefinition::assemble(
            item_prefix,
            GenericsDefinition::parse_where_clauses(stream),
            EnumDefinition::parse_variants(stream),
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
        let input = quote! { pub struct Example<T> where T: Clone (pub T, bool) };
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
        let input = quote! { enum Example { A(u8), B, C = 0usize } };
        let mut iter = input.into_iter().peekable();
        let definition = EnumDefinition::parse(&mut iter);

        assert_eq!(definition.name.to_string(), "Example");
        assert_eq!(definition.variants.len(), 3);
        assert_eq!(definition.variants[0].fields.len(), 1);
        assert!(definition.variants[1].fields.is_empty());
        assert!(definition.variants[2].discriminant.is_some());
    }
}
