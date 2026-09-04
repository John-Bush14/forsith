use std::iter::{Peekable, once};

use proc_macro::{Delimiter::{self, Parenthesis}, Group, Ident, Literal, Punct, Spacing, TokenStream, TokenTree};

macro_rules! quote {
    ($($tt:tt)*) => {{
        #[allow(unused_mut)]
        let mut tokens = TokenStream::new();
        $(
            tokens.extend(quote_tree!($tt));
        )*
        tokens
    }};
}

macro_rules! quote_tree {
    ((@ $($tt:tt)*)) => {[$($tt)*]};
    ($ident:ident) => {{
        use proc_macro::{Ident, Span};
        [Ident::new(stringify!($ident), Span::call_site())]
    }};
    (($($tt:tt)*)) => {{
        use proc_macro::{Group, Delimiter};
        [Group::new(Delimiter::Parenthesis, quote!($($tt)*))]
    }};
    ({$($tt:tt)*}) => {{
        use proc_macro::{Group, Delimiter};
        [Group::new(Delimiter::Brace, quote!($($tt)*))]
    }};
    ([$($tt:tt)*]) => {{
        use proc_macro::{Group, Delimiter};
        [Group::new(Delimiter::Bracket, quote!($($tt)*))]
    }};
    ($lit:literal) => {{
        use std::any::Any;
        use proc_macro::Literal;
        if ($lit).type_id() == "".type_id() {[Literal::string($lit)]}
        else {panic!("Unsupported literal type: {:?}", stringify!($lit))}
    }};
    ($punct:tt) => {{
        use proc_macro::{Punct, Spacing};
        let puncts = stringify!($punct);
        puncts
            .chars()
            .enumerate()
            .map(|(i, c)| Punct::new(c, if i == puncts.len() - 1 {Spacing::Alone} else {Spacing::Joint}))
    }};
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ItemType {
    Struct,
    Enum,
    Union,
    Trait,
    Function,
    Module,
    Constant,
    Static,
    TypeAlias,
}

#[derive(Debug)]
pub enum Generic {
    Lifetime(Ident),
    Type(Ident, TokenStream),
}

#[derive(Debug)]
pub struct Item {
    ty: ItemType,
    name: Ident,
    generics: Vec<Generic>,
}

#[derive(Debug)]
pub struct Attribute {
    pub name: Ident,
    pub args: Option<Group>,
}

impl Attribute {
    pub fn name(&self) -> &Ident {&self.name}
    #[allow(dead_code)]
    pub fn args(&self) -> Option<&Group> {self.args.as_ref()}
}

impl Item {
    pub fn ty(&self) -> &ItemType {&self.ty}
    pub fn name(&self) -> &Ident {&self.name}
}

pub fn parse_attribute_group(input: &mut impl Iterator<Item = TokenTree>) -> Attribute {
    let att = match input.next() {
        Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Bracket => group,
        t => panic!("Expected [...] after `#` in enum variants, found `{:?}`", t),
    };

    let mut att = att.stream().into_iter().peekable();

    let name = match att.next() {
        Some(TokenTree::Ident(ident)) => ident,
        t => panic!("Expected ident after `#` in attribute, found `{:?}`", t),
    };

    let args = match att.next() {
        Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Parenthesis => Some(group),
        None => None,
        t => panic!("Expected (...) after `#ident` in attribute, found `{:?}`", t),
    };

    Attribute { name, args }
}

pub fn parse_struct_fields(input: &mut impl Iterator<Item = TokenTree>) -> Vec<(TokenTree, TokenStream, Vec<Attribute>)> {
    let mut fields = Vec::new();

    let group = match input.next() {
        Some(TokenTree::Group(group)) => group,
        t => panic!("Expected group of struct fields, found `{:?}`", t),
    };

    let mut field = (None, None, Vec::new());

    let mut iter = group.stream().into_iter().peekable();

    if group.delimiter() == Delimiter::Parenthesis {
        let mut i = 0;
        let mut ty_tokens = TokenStream::new();

        for item in iter.by_ref() {
            match item {
                TokenTree::Punct(ref punct) if punct.as_char() == ',' => {
                    fields.push((TokenTree::Literal(Literal::usize_unsuffixed(i)), ty_tokens, std::mem::take(&mut field.2)));
                    i += 1;
                    ty_tokens = TokenStream::new();
                },
                i => ty_tokens.extend(once(i)),
            }
        }

        field.1 = Some(ty_tokens);
        fields.push((TokenTree::Literal(Literal::usize_unsuffixed(i)), field.1.take().expect("Last field has no type"), std::mem::take(&mut field.2)));

        return fields;
    }

    while let Some(token) = iter.next() {
        match token {
            TokenTree::Ident(ident) => field.0 = Some(ident),
            TokenTree::Punct(punct) if punct.as_char() == ':' => {
                let mut ty_tokens = TokenStream::new();
                let mut nested = 0;
                while let Some(item) = iter.peek() {
                    if let TokenTree::Punct(punct) = item {
                        match punct.as_char() {
                            '<' => nested += 1,
                            '>' => nested -= 1,
                            ',' if nested == 0 => break,
                            _ => {}
                        };
                    }
                    ty_tokens.extend(once(iter.next().unwrap()));
                }
                field.1 = Some(ty_tokens);
            },
            TokenTree::Punct(punct) if punct.as_char() == ',' => {
                fields.push((TokenTree::Ident(field.0.take().expect("Expected ident before comma")), field.1.take().expect("Field without type"), std::mem::take(&mut field.2)));
            },
            TokenTree::Punct(punct) if punct.as_char() == '#' => {
                field.2.push(parse_attribute_group(&mut iter));
            },
            t => panic!("Expected ident or comma in struct fields, found `{:?}`", t),
        }
    }

    if !matches!(field, (None, None, _)) || !field.2.is_empty() {
        fields.push((TokenTree::Ident(field.0.take().expect("No field Ident?")), field.1.take().expect("Last field has no type"), std::mem::take(&mut field.2)));
    }

    fields
}

pub fn parse_enum_variants(input: &mut impl Iterator<Item = TokenTree>) -> Vec<(Ident, Option<Group>, Vec<Attribute>)> {
    let mut variants = Vec::new();

    let group = match input.next() {
        Some(TokenTree::Group(group)) => group,
        t => panic!("Expected group of enum variants, found `{:?}`", t),
    };

    let mut variant = (None, None, Vec::new());

    let mut iter = group.stream().into_iter();
    while let Some(token) = iter.next() {
        match token {
            TokenTree::Ident(ident) => variant.0 = Some(ident),
            TokenTree::Group(group) => variant.1 = Some(group),
            TokenTree::Punct(punct) if punct.as_char() == ',' => {
                variants.push((variant.0.take().expect("Expected ident before comma"), variant.1.take(), std::mem::take(&mut variant.2)));
            },
            TokenTree::Punct(punct) if punct.as_char() == '#' => {
                variant.2.push(parse_attribute_group(&mut iter));
            },
            t => panic!("Expected ident or comma in enum variants, found `{:?}`", t),
        }
    }

    if !matches!(variant, (None, None, _)) || !variant.2.is_empty() {
        variants.push((variant.0.take().expect("No variant Ident?"), variant.1.take(), std::mem::take(&mut variant.2)));
    }

    variants
}

pub fn impl_item(item: &Item, r#trait: Option<TokenStream>, body: TokenStream) -> TokenStream {
    let generic_def = item.generics.iter().map(|generic| {
        match generic {
            Generic::Type(name, constraints) => quote!((@ name.clone()): (@ constraints.clone()),),
            Generic::Lifetime(name) => TokenStream::from_iter([TokenTree::Punct(Punct::new('\'', Spacing::Joint)), TokenTree::Ident(name.clone()), TokenTree::Punct(Punct::new(',', Spacing::Alone))].into_iter()),
        }
    }).collect::<TokenStream>();

    let generic_use = item.generics.iter().map(|generic|
        match generic {
            Generic::Type(name, _) => quote!((@ name.clone()),),
            Generic::Lifetime(name) => TokenStream::from_iter([TokenTree::Punct(Punct::new('\'', Spacing::Joint)), TokenTree::Ident(name.clone()), TokenTree::Punct(Punct::new(',', Spacing::Alone))].into_iter()),
        }
    ).collect::<TokenStream>();

    let mut impl_item = quote!(impl<(@ generic_def)>);

    if let Some(r#trait) = r#trait {
        impl_item.extend(quote!( (@ r#trait) for));
    }

    impl_item.extend(quote!(
        (@ item.name().clone())<(@ generic_use)> {
            (@ body)
        }
    ));

    impl_item
}

pub fn parse_item(input: &mut Peekable<impl Iterator<Item = TokenTree>>) -> Item {
    if let Some(TokenTree::Ident(ident)) = input.peek() && ident.to_string() == "pub" {
        let _ = input.next();
        if let Some(TokenTree::Group(group)) = input.peek() && group.delimiter() == Parenthesis {
            let _ = input.next();
        }
    }

    let item_ident = input.next().expect("Expected item type, found None");

    let ty = match item_ident.to_string().as_str() {
        "struct" => ItemType::Struct,
        "enum" => ItemType::Enum,
        "union" => ItemType::Union,
        "trait" => ItemType::Trait,
        "fn" => ItemType::Function,
        "mod" => ItemType::Module,
        "const" => ItemType::Constant,
        "static" => ItemType::Static,
        "type" => ItemType::TypeAlias,
        _ => panic!("Expected item type, found `{}`", item_ident),
    };

    let name = match input.next() {
        Some(TokenTree::Ident(ident)) => ident,
        t => panic!("Expected item name after `{}`, found `{:?}`", item_ident, t),
    };

    let mut generics: Vec<Generic> = Vec::new();
    if let Some(TokenTree::Punct(punct)) = input.peek() && punct.as_char() == '<' {
        let _ = input.next();

        loop {
            let generic_ident = match input.next() {
                Some(TokenTree::Ident(ident)) => ident,
                Some(TokenTree::Punct(punct)) if punct.as_char() == '>' => break,
                Some(TokenTree::Punct(punct)) if punct.as_char() == '\'' => {
                    let lifetime_ident = match input.next() {
                        Some(TokenTree::Ident(ident)) => ident,
                        t => panic!("Expected lifetime name after `'`, found `{:?}`", t),
                    };
                    generics.push(Generic::Lifetime(lifetime_ident));

                    if let Some(TokenTree::Punct(punct)) = input.peek() && punct.as_char() == ',' {
                        let _ = input.next();
                    }

                    continue
                },
                None => panic!("Expected Some after `<` in generics, found None"),
                tt => panic!("Expected ident or `>` in generics, found `{:?}`", tt),
            };

            let mut constraints = TokenStream::new();
            if let Some(TokenTree::Punct(punct)) = input.next() && punct.as_char() == ':' {
                let mut nested = 0;
                while let Some(item) = input.peek() {
                    if let TokenTree::Punct(punct) = item {
                        match punct.as_char() {
                            '<' => nested += 1,
                            '>' => {
                                if nested == 0 {break;}
                                nested -= 1;
                            },
                            ',' if nested == 0 => {let _ = input.next(); break},
                            _ => {}
                        };
                    }
                    constraints.extend(once(input.next().unwrap()));
                }
            }
            generics.push(Generic::Type(generic_ident, constraints));
        }
    };

    Item { ty, name, generics }
}

