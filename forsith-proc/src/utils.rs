use std::iter::{Peekable, once};

use proc_macro::{Delimiter::Parenthesis, Group, Ident, TokenStream, TokenTree};

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
pub struct Item {
    ty: ItemType,
    name: Ident,
    generics: Vec<(Ident, TokenStream)>,
}

impl Item {
    pub fn ty(&self) -> &ItemType {&self.ty}
    pub fn name(&self) -> &Ident {&self.name}
}

pub fn parse_enum_variants(input: &mut impl Iterator<Item = TokenTree>) -> Vec<(Ident, Option<Group>)> {
    let mut variants = Vec::new();

    let group = match input.next() {
        Some(TokenTree::Group(group)) => group,
        t => panic!("Expected group of enum variants, found `{:?}`", t),
    };

    let mut variant = (None, None);

    let mut iter = group.stream().into_iter();
    while let Some(token) = iter.next() {
        match token {
            TokenTree::Ident(ident) => variant.0 = Some(ident),
            TokenTree::Group(group) => variant.1 = Some(group),
            TokenTree::Punct(punct) if punct.as_char() == ',' => {
                variants.push((variant.0.take().expect("Expected ident before comma"), variant.1.take()))
            },
            TokenTree::Punct(punct) if punct.as_char() == '#' => {
                let att = iter.next();
                assert!(matches!(att, Some(TokenTree::Group(_))), "Expected group after `#` in enum variants, found `{:?}`", att);
            },
            t => panic!("Expected ident or comma in enum variants, found `{:?}`", t),
        }
    }

    if !matches!(variant, (None, None)) {
        variants.push((variant.0.take().expect("varant group without variant ident?"), variant.1.take()))
    }

    variants
}

pub fn impl_item(item: &Item, body: TokenStream) -> TokenStream {
    let generic_def = item.generics.iter().map(|(name, constraints)| {
        quote!(
            (@ name.clone()): (@ constraints.clone()),
        )
    }).collect::<TokenStream>();

    let generic_use = item.generics.iter().map(|(name, _)|
        quote!((@ name.clone()),)
    ).collect::<TokenStream>();

    quote!(
        impl<(@ generic_def)> (@ item.name().clone())<(@ generic_use)> {
            (@ body)
        }
    )
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

    let mut generics: Vec<(Ident, TokenStream)> = Vec::new();
    if let Some(TokenTree::Punct(punct)) = input.peek() && punct.as_char() == '<' {
        let _ = input.next();

        loop {
            let generic_ident = match input.next() {
                Some(TokenTree::Ident(ident)) => ident,
                Some(TokenTree::Punct(punct)) if punct.as_char() == '>' => break,
                None => panic!("Expected Some after `<` in generics, found None"),
                tt => panic!("Expected ident or `>` in generics, found `{:?}`", tt),
            };

            let mut constraints = TokenStream::new();
            if let Some(TokenTree::Punct(punct)) = input.peek() && punct.as_char() == ':' {
                while let Some(item) = input.peek() {
                    if let TokenTree::Punct(punct) = item && punct.as_char() == '>' {
                        break;
                    }
                    constraints.extend(once(input.next().unwrap()));
                }
            }
            generics.push((generic_ident, constraints));
        }
    };

    Item { ty, name, generics }
}

