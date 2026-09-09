use std::{ffi::CString, fmt::Display, ops::{Deref, Index, IndexMut}};

#[cfg(feature = "in_proc_macro")]
extern crate proc_macro;

pub mod quote;

#[derive(Clone, Debug, Default)]
pub struct TokenStream(Vec<TokenTree>);

impl TokenStream {
    #[must_use]
    pub fn new() -> Self {Self::default()}

    #[must_use]
    pub const fn is_empty(&self) -> bool {self.0.is_empty()}
    #[must_use]
    pub const fn len(&self) -> usize {self.0.len()}
}

impl Index<usize> for TokenStream {
    type Output = TokenTree;

    fn index(&self, index: usize) -> &Self::Output {&self.0[index]}
}

impl IndexMut<usize> for TokenStream {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {&mut self.0[index]}
}

#[cfg(feature = "in_proc_macro")]
impl From<proc_macro::TokenStream> for TokenStream {
    fn from(ts: proc_macro::TokenStream) -> Self {
        Self(ts.into_iter().map(Into::into).collect())
    }
}

#[cfg(feature = "in_proc_macro")]
impl From<TokenStream> for proc_macro::TokenStream {
    fn from(sts: TokenStream) -> Self {
        let mut pcts =  Self::new();
        pcts.extend(sts.0.into_iter().map(Into::<proc_macro::TokenTree>::into));
        pcts
    }
}

impl From<Vec<TokenTree>> for TokenStream {
    fn from(value: Vec<TokenTree>) -> Self {Self(value)}
}

impl Display for TokenStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for tt in &self.0 {write!(f, "{tt}")?;}
        Ok(())
    }
}

impl FromIterator<Self> for TokenStream {
    fn from_iter<I: IntoIterator<Item = Self>>(iter: I) -> Self {
        let mut ts = Self::default();
        ts.extend(iter);
        ts
    }
}

impl IntoIterator for TokenStream {
    type Item = TokenTree;
    type IntoIter = std::vec::IntoIter<TokenTree>;

    fn into_iter(self) -> Self::IntoIter {self.0.into_iter()}
}

impl Extend<Self> for TokenStream {
    fn extend<I: IntoIterator<Item = Self>>(&mut self, iter: I) {
        for ts in iter {self.0.extend(ts);}
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
            proc_macro::TokenTree::Group(g) => Self::Group(g.into()),
            proc_macro::TokenTree::Ident(i) => Self::Ident(i.into()),
            proc_macro::TokenTree::Punct(p) => Self::Punct(p.into()),
            proc_macro::TokenTree::Literal(l) => Self::Literal(l.into()),
        }
    }
}

#[cfg(feature = "in_proc_macro")]
impl From<TokenTree> for proc_macro::TokenTree {
    fn from(tt: TokenTree) -> Self {
        match tt {
            TokenTree::Group(g) => Self::Group(g.into()),
            TokenTree::Ident(i) => Self::Ident(i.into()),
            TokenTree::Punct(p) => Self::Punct(p.into()),
            TokenTree::Literal(l) => Self::Literal(l.into()),
        }
    }
}

impl Display for TokenTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Group(g) => write!(f, "{g}"),
            Self::Ident(i) => write!(f, "{i}"),
            Self::Punct(p) => write!(f, "{p}"),
            Self::Literal(l) => write!(f, "{l}"),
        }
    }
}

impl Extend<TokenTree> for TokenStream {
    fn extend<T: IntoIterator<Item = TokenTree>>(&mut self, iter: T) {
        self.0.extend(iter);
    }
}

impl FromIterator<TokenTree> for TokenStream {
    fn from_iter<I: IntoIterator<Item = TokenTree>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
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

#[cfg(feature = "in_proc_macro")]
impl From<Ident> for proc_macro::Ident {
    fn from(i: Ident) -> Self {
        Self::new(&i.0, proc_macro::Span::call_site())
    }
}

impl Extend<Ident> for TokenStream {
    fn extend<T: IntoIterator<Item = Ident>>(&mut self, iter: T) {
        self.0.extend(iter.into_iter().map(TokenTree::Ident));
    }
}

impl Ident {
    #[must_use]
    pub fn new(name: &str) -> Self {Self(name.to_string())}
}

impl Display for Ident {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Deref for Ident {
    type Target = str;

    fn deref(&self) -> &Self::Target {&self.0}
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Punct {
    char: PunctChar,
    joint: bool,
}

#[cfg(feature = "in_proc_macro")]
#[allow(clippy::fallible_impl_from)]
impl From<proc_macro::Punct> for Punct {
    fn from(p: proc_macro::Punct) -> Self {
        Self {
            char: p.as_char().try_into().expect("Failed to convert proc_macro::Punct to PunctChar"),
            joint: p.spacing() == proc_macro::Spacing::Joint,
        }
    }
}

#[cfg(feature = "in_proc_macro")]
impl From<Punct> for proc_macro::Punct {
    fn from(p: Punct) -> Self {
        let spacing = if p.joint {proc_macro::Spacing::Joint} else {proc_macro::Spacing::Alone};
        Self::new(p.char.into(), spacing)
    }
}

impl Extend<Punct> for TokenStream {
    fn extend<T: IntoIterator<Item = Punct>>(&mut self, iter: T) {
        self.0.extend(iter.into_iter().map(TokenTree::Punct));
    }
}

impl Punct {
    #[must_use]
    pub const fn new(char: PunctChar, joint: bool) -> Self {Self {char, joint}}

    #[must_use]
    pub const fn char(&self) -> PunctChar {self.char}
    #[must_use]
    pub const fn joint(&self) -> bool {self.joint}
}

#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub enum PunctChar {
    Comma,
    Qoute,
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
}

impl TryFrom<char> for PunctChar {
    type Error = ();

    fn try_from(c: char) -> Result<Self, Self::Error> {
        #[allow(clippy::enum_glob_use)]
        use PunctChar::*;
        Ok(match c {
            ',' => Comma,
            ';' => Semicolon,
            ':' => Colon,
            '.' => Dot,
            '+' => Plus,
            '-' => Minus,
            '*' => Star,
            '\'' => Qoute,
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
            _ => return Err(()),
        })
    }
}

impl From<PunctChar> for char {
    fn from(pc: PunctChar) -> Self {
        #[allow(clippy::enum_glob_use)]
        use PunctChar::*;
        match pc {
            Comma => ',',
            Semicolon => ';',
            Colon => ':',
            Dot => '.',
            Plus => '+',
            Minus => '-',
            Star => '*',
            Slash => '/',
            Percent => '%',
            Caret => '^',
            Ampersand => '&',
            Pipe => '|',
            Bang => '!',
            Tilde => '~',
            Hash => '#',
            Equal => '=',
            LessThan => '<',
            GreaterThan => '>',
            Qoute => '\'',
        }
    }
}

impl Display for Punct {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.joint {
            write!(f, "{}", char::from(self.char))
        } else {
            write!(f, "{} ", char::from(self.char))
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Literal {
    Char(char),
    Integer(usize, Option<IntegerSuffix>),
    Float(f64, Option<FloatSuffix>),
    Str(String),
    ByteStr(Vec<u8>),
    CStr(CString),
}

#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub enum IntegerSuffix {
    U8,
    U16,
    U32,
    U64,
    Usize,
    I8,
    I16,
    I32,
    I64,
    Isize,
}

impl IntegerSuffix {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::U8 => "u8",
            Self::U16 => "u16",
            Self::U32 => "u32",
            Self::U64 => "u64",
            Self::Usize => "usize",
            Self::I8 => "i8",
            Self::I16 => "i16",
            Self::I32 => "i32",
            Self::I64 => "i64",
            Self::Isize => "isize",
        }
    }

    #[must_use]
    pub const fn variants() -> &'static [Self] {
        &[
            Self::U8,
            Self::U16,
            Self::U32,
            Self::U64,
            Self::Usize,
            Self::I8,
            Self::I16,
            Self::I32,
            Self::I64,
            Self::Isize,
        ]
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub enum FloatSuffix {
    F32,
    F64,
}

impl FloatSuffix {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::F32 => "f32",
            Self::F64 => "f64",
        }
    }

    #[must_use]
    pub const fn variants() -> &'static [Self] {
        &[Self::F32, Self::F64]
    }
}

#[cfg(feature = "in_proc_macro")]
#[allow(clippy::fallible_impl_from)]
impl From<proc_macro::Literal> for Literal {
    fn from(l: proc_macro::Literal) -> Self {
        for suffix in IntegerSuffix::variants() {
            if l.to_string().ends_with(suffix.as_str()) {
                let value = l.to_string()[..l.to_string().len() - suffix.as_str().len()].to_string();
                if let Ok(i) = value.parse::<usize>() {
                    return Self::Integer(i, Some(*suffix));
                }
            }
        }

        for suffix in FloatSuffix::variants() {
            if l.to_string().ends_with(suffix.as_str()) {
                let value = l.to_string()[..l.to_string().len() - suffix.as_str().len()].to_string();
                if let Ok(i) = value.parse::<f64>() {
                    return Self::Float(i, Some(*suffix));
                }
            }
        }

        let s = l.to_string();
        #[allow(clippy::option_if_let_else)]
        if let Ok(c) = s.parse::<char>() {
            Self::Char(c)
        } else if let Ok(i) = l.to_string().parse::<usize>() {
            Self::Integer(i, None)
        } else if let Ok(f) = l.to_string().parse::<f64>() {
            Self::Float(f, None)
        } else if s.starts_with('"') && s.ends_with('"') {
            Self::Str(s[1..s.len() - 1].to_string())
        } else if s.starts_with("b\"") && s.ends_with('"') {
            Self::ByteStr(s.as_bytes()[2..s.len() - 1].to_vec())
        } else if s.starts_with("c\"") && s.ends_with('"') {
            let cstr = CString::new(&s[2..s.len() - 1]).expect("Failed to create CString");
            Self::CStr(cstr)
        } else {
            panic!("Couldn't parse std proc_macro Literal: {s}");
        }
    }
}

#[cfg(feature = "in_proc_macro")]
impl From<Literal> for proc_macro::Literal {
    fn from(l: Literal) -> Self {
        match l {
            Literal::Char(c) => Self::character(c),
            Literal::Integer(i, Some(suffix)) => {
                match suffix {
                    IntegerSuffix::U8 => Self::u8_suffixed(i.try_into().expect("Literal with suffix u8 must be valid u8")),
                    IntegerSuffix::U16 => Self::u16_suffixed(i.try_into().expect("Literal with suffix u16 must be valid u16")),
                    IntegerSuffix::U32 => Self::u32_suffixed(i.try_into().expect("Literal with suffix u32 must be valid u32")),
                    IntegerSuffix::U64 => Self::u64_suffixed(i as u64),
                    IntegerSuffix::Usize => Self::usize_suffixed(i),
                    IntegerSuffix::I8 => Self::i8_suffixed(i.try_into().expect("Literal with suffix i8 must be valid i8")),
                    IntegerSuffix::I16 => Self::i16_suffixed(i.try_into().expect("Literal with suffix i16 must be valid i16")),
                    IntegerSuffix::I32 => Self::i32_suffixed(i.try_into().expect("Literal with suffix i32 must be valid i32")),
                    IntegerSuffix::I64 => Self::i64_suffixed(i.try_into().expect("Literal with suffix i64 must be valid i64")),
                    IntegerSuffix::Isize => Self::isize_suffixed(i.try_into().expect("Literal with suffix isize must be valid isize"))
                }
            }
            Literal::Integer(i, None) => Self::usize_unsuffixed(i),
            Literal::Float(f, Some(suffix)) => {
                #[allow(clippy::cast_possible_truncation)]
                match suffix {
                    FloatSuffix::F32 => Self::f32_suffixed(f as f32),
                    FloatSuffix::F64 => Self::f64_suffixed(f),
                }
            }
            Literal::Float(f, None) => Self::f64_unsuffixed(f),
            Literal::Str(s) => Self::string(&s),
            Literal::ByteStr(bs) => Self::byte_string(&bs),
            Literal::CStr(cstr) => {
                let s = cstr.to_str().expect("Failed to convert CString to str");
                Self::string(s)
            }
        }
    }
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Char(c) => write!(f, "'{c}'"),
            Self::Integer(i, Some(suffix)) => write!(f, "{}{}", i, suffix.as_str()),
            Self::Integer(i, None) => write!(f, "{i}"),
            Self::Float(fl, Some(suffix)) => write!(f, "{}{}", fl, suffix.as_str()),
            Self::Float(fl, None) => write!(f, "{fl}"),
            Self::Str(s) => write!(f, "\"{s}\""),
            Self::ByteStr(bs) => write!(f, "b\"{}\"", String::from_utf8_lossy(bs)),
            Self::CStr(cstr) => write!(f, "c\"{}\"", cstr.to_str().expect("Failed to convert CString to str")),
        }
    }
}

impl Extend<Literal> for TokenStream {
    fn extend<T: IntoIterator<Item = Literal>>(&mut self, iter: T) {
        self.0.extend(iter.into_iter().map(TokenTree::Literal));
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

#[cfg(feature = "in_proc_macro")]
impl From<Group> for proc_macro::Group {
    fn from(g: Group) -> Self {
        let mut group = Self::new(g.delimiter().into(), g.stream.into());
        group.set_span(proc_macro::Span::call_site());
        group
    }
}

impl  Display for Group {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (open, close) = match self.delimiter {
            Delimiter::Parenthesis => ('(', ')'),
            Delimiter::Brace => ('{', '}'),
            Delimiter::Bracket => ('[', ']'),
            Delimiter::None => (' ', ' '),
        };
        write!(f, "{}{}{}", open, self.stream, close)
    }
}

impl Extend<Group> for TokenStream {
    fn extend<T: IntoIterator<Item = Group>>(&mut self, iter: T) {
        self.0.extend(iter.into_iter().map(TokenTree::Group));
    }
}

impl Group {
    #[must_use]
    pub const fn new(delimiter: Delimiter, stream: TokenStream) -> Self {Self {delimiter, stream}}

    #[must_use]
    pub const fn stream(&self) -> &TokenStream {&self.stream}
    #[must_use]
    pub fn take_stream(self) -> TokenStream {self.stream}
    #[must_use]
    pub const fn delimiter(&self) -> Delimiter {self.delimiter}
}

#[derive(Clone, Debug, PartialEq, Eq, Copy)]
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
            proc_macro::Delimiter::Parenthesis => Self::Parenthesis,
            proc_macro::Delimiter::Brace => Self::Brace,
            proc_macro::Delimiter::Bracket => Self::Bracket,
            proc_macro::Delimiter::None => Self::None,
        }
    }
}

#[cfg(feature = "in_proc_macro")]
impl From<Delimiter> for proc_macro::Delimiter {
    fn from(d: Delimiter) -> Self {
        match d {
            Delimiter::Parenthesis => Self::Parenthesis,
            Delimiter::Brace => Self::Brace,
            Delimiter::Bracket => Self::Bracket,
            Delimiter::None => Self::None,
        }
    }
}
