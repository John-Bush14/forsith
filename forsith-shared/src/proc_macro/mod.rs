use std::ffi::CString;

#[cfg(feature = "in_proc_macro")]
extern crate proc_macro;

#[derive(Clone, Debug)]
pub struct TokenStream {
    pub tokens: Vec<TokenTree>
}

#[cfg(feature = "in_proc_macro")]
impl From<proc_macro::TokenStream> for TokenStream {
    fn from(ts: proc_macro::TokenStream) -> Self {
        let mut tokens = Vec::new();

        for token in ts {
            tokens.push(token.into());
        }

        Self { tokens }
    }
}

#[derive(Clone, Debug)]
pub struct Ident(String);

#[cfg(feature = "in_proc_macro")]
impl From<proc_macro::Ident> for Ident {
    fn from(i: proc_macro::Ident) -> Self {
        Self(i.to_string())
    }
}

#[derive(Clone, Debug)]
pub enum TokenTree {
    Group(Group),
    Ident(Ident),
    Punct(Punct),
    Literal(Literal),
}

#[cfg(feature = "in_proc_macro")]
impl From<proc_macro::TokenTree> for TokenTree {
    fn from(tt: proc_macro::TokenTree) -> Self {
        match tt {
            proc_macro::TokenTree::Group(g) => TokenTree::Group(g.into()),
            proc_macro::TokenTree::Ident(i) => TokenTree::Ident(i.into()),
            proc_macro::TokenTree::Punct(p) => TokenTree::Punct(p.into()),
            proc_macro::TokenTree::Literal(l) => TokenTree::Literal(l.into()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Punct {
    char: PunctChar,
    joint: bool,
}

#[cfg(feature = "in_proc_macro")]
impl From<proc_macro::Punct> for Punct {
    fn from(p: proc_macro::Punct) -> Self {
        Self {
            char: p.as_char().into(),
            joint: p.spacing() == proc_macro::Spacing::Joint,
        }
    }
}

#[derive(Clone, Debug)]
pub enum PunctChar {
    Comma,
    Semicolon,
    Colon,
    Dot,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Caret,
    Ampersand,
    Pipe,
    Bang,
    Tilde,
    Hash,
    Equal,
    LessThan,
    GreaterThan,
    GreaterEqual,
    LessEqual,
    NotEqual,
}

impl From<char> for PunctChar {
    fn from(c: char) -> Self {
        use PunctChar::*;
        match c {
            ',' => Comma,
            ';' => Semicolon,
            ':' => Colon,
            '.' => Dot,
            '+' => Plus,
            '-' => Minus,
            '*' => Star,
            '/' => Slash,
            '%' => Percent,
            '^' => Caret,
            '&' => Ampersand,
            '|' => Pipe,
            '!' => Bang,
            '~' => Tilde,
            '=' => Equal,
            '#' => Hash,
            '<' => LessThan,
            '>' => GreaterThan,
            _ => panic!("Unsupported punctuation: {}", c),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Literal {
    Char(char),
    Integer(usize, Option<String>),
    Float(f64, Option<String>),
    Str(String),
    ByteStr(Vec<u8>),
    CStr(CString),
}

#[cfg(feature = "in_proc_macro")]
impl From<proc_macro::Literal> for Literal {
    fn from(l: proc_macro::Literal) -> Self {
        for suffix in &["u8", "u16", "u32", "u64", "usize", "i8", "i16", "i32", "i64", "isize"] {
            if l.to_string().ends_with(suffix) {
                let value = l.to_string()[..l.to_string().len() - suffix.len()].to_string();
                if let Ok(i) = value.parse::<usize>() {
                    return Literal::Integer(i, Some(suffix.to_string()));
                }
            }
        }

        for suffix in &["f32", "f64"] {
            if l.to_string().ends_with(suffix) {
                let value = l.to_string()[..l.to_string().len() - suffix.len()].to_string();
                if let Ok(i) = value.parse::<f64>() {
                    return Literal::Float(i, Some(suffix.to_string()));
                }
            }
        }

        let s = l.to_string();
        if let Ok(c) = s.parse::<char>() {
            Literal::Char(c)
        } else if let Ok(i) = l.to_string().parse::<usize>() {
            Literal::Integer(i, None)
        } else if let Ok(f) = l.to_string().parse::<f64>() {
            Literal::Float(f, None)
        } else if s.starts_with('"') && s.ends_with('"') {
            Literal::Str(s[1..s.len() - 1].to_string())
        } else if s.starts_with("b\"") && s.ends_with('"') {
            Literal::ByteStr(s.as_bytes()[2..s.len() - 1].to_vec())
        } else if s.starts_with("c\"") && s.ends_with('"') {
            let cstr = CString::new(&s[2..s.len() - 1]).expect("Failed to create CString");
            Literal::CStr(cstr)
        } else {
            panic!("Unsupported literal: {}", s);
        }
    }
}

#[derive(Clone, Debug)]
pub struct Group {
    pub delimiter: Delimiter,
    pub stream: TokenStream,
}

#[cfg(feature = "in_proc_macro")]
impl From<proc_macro::Group> for Group {
    fn from(g: proc_macro::Group) -> Self {
        Self {
            delimiter: g.delimiter().into(),
            stream: g.stream().into(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Delimiter {
    Parenthesis,
    Brace,
    Bracket,
    None,
}

#[cfg(feature = "in_proc_macro")]
impl From<proc_macro::Delimiter> for Delimiter {
    fn from(d: proc_macro::Delimiter) -> Self {
        match d {
            proc_macro::Delimiter::Parenthesis => Delimiter::Parenthesis,
            proc_macro::Delimiter::Brace => Delimiter::Brace,
            proc_macro::Delimiter::Bracket => Delimiter::Bracket,
            proc_macro::Delimiter::None => Delimiter::None,
        }
    }
}
