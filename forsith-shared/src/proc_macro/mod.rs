use std::{ffi::CString, fmt::Display, ops::{Deref, Index, IndexMut}};

#[cfg(feature = "in_proc_macro")]
extern crate proc_macro;

#[derive(Clone, Debug, Default)]
pub struct TokenStream(Vec<TokenTree>);

impl TokenStream {
    pub fn new() -> Self {Self::default()}

    pub fn is_empty(&self) -> bool {self.0.is_empty()}
    pub fn len(&self) -> usize {self.0.len()}

    pub fn inner(&self) -> &[TokenTree] {&self.0}
    pub fn inner_mut(&mut self) -> &mut Vec<TokenTree> {&mut self.0}
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
        let mut pcts =  proc_macro::TokenStream::new();
        pcts.extend(sts.0.into_iter().map(Into::<proc_macro::TokenTree>::into));
        pcts
    }
}

impl From<Vec<TokenTree>> for TokenStream {
    fn from(value: Vec<TokenTree>) -> Self {Self(value)}
}

impl Display for TokenStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for tt in &self.0 {write!(f, "{}", tt)?;}
        Ok(())
    }
}

impl FromIterator<TokenStream> for TokenStream {
    fn from_iter<I: IntoIterator<Item = TokenStream>>(iter: I) -> Self {
        let mut ts = TokenStream::default();
        ts.extend(iter);
        ts
    }
}

impl IntoIterator for TokenStream {
    type Item = TokenTree;
    type IntoIter = std::vec::IntoIter<TokenTree>;

    fn into_iter(self) -> Self::IntoIter {self.0.into_iter()}
}

impl Extend<TokenStream> for TokenStream {
    fn extend<I: IntoIterator<Item = TokenStream>>(&mut self, iter: I) {
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
            proc_macro::TokenTree::Group(g) => TokenTree::Group(g.into()),
            proc_macro::TokenTree::Ident(i) => TokenTree::Ident(i.into()),
            proc_macro::TokenTree::Punct(p) => TokenTree::Punct(p.into()),
            proc_macro::TokenTree::Literal(l) => TokenTree::Literal(l.into()),
        }
    }
}

#[cfg(feature = "in_proc_macro")]
impl From<TokenTree> for proc_macro::TokenTree {
    fn from(tt: TokenTree) -> Self {
        match tt {
            TokenTree::Group(g) => proc_macro::TokenTree::Group(g.into()),
            TokenTree::Ident(i) => proc_macro::TokenTree::Ident(i.into()),
            TokenTree::Punct(p) => proc_macro::TokenTree::Punct(p.into()),
            TokenTree::Literal(l) => proc_macro::TokenTree::Literal(l.into()),
        }
    }
}

impl Display for TokenTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenTree::Group(g) => write!(f, "{}", g),
            TokenTree::Ident(i) => write!(f, "{}", i),
            TokenTree::Punct(p) => write!(f, "{}", p),
            TokenTree::Literal(l) => write!(f, "{}", l),
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
        proc_macro::Ident::new(&i.0, proc_macro::Span::call_site())
    }
}

impl Extend<Ident> for TokenStream {
    fn extend<T: IntoIterator<Item = Ident>>(&mut self, iter: T) {
        self.0.extend(iter.into_iter().map(TokenTree::Ident));
    }
}

impl Ident {
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
impl From<proc_macro::Punct> for Punct {
    fn from(p: proc_macro::Punct) -> Self {
        Self {
            char: p.as_char().into(),
            joint: p.spacing() == proc_macro::Spacing::Joint,
        }
    }
}

#[cfg(feature = "in_proc_macro")]
impl From<Punct> for proc_macro::Punct {
    fn from(p: Punct) -> Self {
        let spacing = if p.joint {proc_macro::Spacing::Joint} else {proc_macro::Spacing::Alone};
        proc_macro::Punct::new(p.char.into(), spacing)
    }
}

impl Extend<Punct> for TokenStream {
    fn extend<T: IntoIterator<Item = Punct>>(&mut self, iter: T) {
        self.0.extend(iter.into_iter().map(TokenTree::Punct));
    }
}

impl Punct {
    pub const fn new(char: PunctChar, joint: bool) -> Self {Self {char, joint}}

    pub const fn char(&self) -> PunctChar {self.char}
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
    GreaterEqual,
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
            _ => panic!("Unsupported punctuation: {}", c),
        }
    }
}

impl From<PunctChar> for char {
    fn from(pc: PunctChar) -> Self {
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
            GreaterEqual => '>',
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
    pub fn as_str(&self) -> &'static str {
        match self {
            IntegerSuffix::U8 => "u8",
            IntegerSuffix::U16 => "u16",
            IntegerSuffix::U32 => "u32",
            IntegerSuffix::U64 => "u64",
            IntegerSuffix::Usize => "usize",
            IntegerSuffix::I8 => "i8",
            IntegerSuffix::I16 => "i16",
            IntegerSuffix::I32 => "i32",
            IntegerSuffix::I64 => "i64",
            IntegerSuffix::Isize => "isize",
        }
    }

    pub fn variants() -> &'static [IntegerSuffix] {
        &[
            IntegerSuffix::U8,
            IntegerSuffix::U16,
            IntegerSuffix::U32,
            IntegerSuffix::U64,
            IntegerSuffix::Usize,
            IntegerSuffix::I8,
            IntegerSuffix::I16,
            IntegerSuffix::I32,
            IntegerSuffix::I64,
            IntegerSuffix::Isize,
        ]
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub enum FloatSuffix {
    F32,
    F64,
}

impl FloatSuffix {
    pub fn as_str(&self) -> &'static str {
        match self {
            FloatSuffix::F32 => "f32",
            FloatSuffix::F64 => "f64",
        }
    }

    pub fn variants() -> &'static [FloatSuffix] {
        &[FloatSuffix::F32, FloatSuffix::F64]
    }
}

#[cfg(feature = "in_proc_macro")]
impl From<proc_macro::Literal> for Literal {
    fn from(l: proc_macro::Literal) -> Self {
        for suffix in IntegerSuffix::variants() {
            if l.to_string().ends_with(suffix.as_str()) {
                let value = l.to_string()[..l.to_string().len() - suffix.as_str().len()].to_string();
                if let Ok(i) = value.parse::<usize>() {
                    return Literal::Integer(i, Some(*suffix));
                }
            }
        }

        for suffix in FloatSuffix::variants() {
            if l.to_string().ends_with(suffix.as_str()) {
                let value = l.to_string()[..l.to_string().len() - suffix.as_str().len()].to_string();
                if let Ok(i) = value.parse::<f64>() {
                    return Literal::Float(i, Some(*suffix));
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

#[cfg(feature = "in_proc_macro")]
impl From<Literal> for proc_macro::Literal {
    fn from(l: Literal) -> Self {
        match l {
            Literal::Char(c) => proc_macro::Literal::character(c),
            Literal::Integer(i, Some(suffix)) => {
                match suffix {
                    IntegerSuffix::U8 => proc_macro::Literal::u8_suffixed(i as u8),
                    IntegerSuffix::U16 => proc_macro::Literal::u16_suffixed(i as u16),
                    IntegerSuffix::U32 => proc_macro::Literal::u32_suffixed(i as u32),
                    IntegerSuffix::U64 => proc_macro::Literal::u64_suffixed(i as u64),
                    IntegerSuffix::Usize => proc_macro::Literal::usize_suffixed(i),
                    IntegerSuffix::I8 => proc_macro::Literal::i8_suffixed(i as i8),
                    IntegerSuffix::I16 => proc_macro::Literal::i16_suffixed(i as i16),
                    IntegerSuffix::I32 => proc_macro::Literal::i32_suffixed(i as i32),
                    IntegerSuffix::I64 => proc_macro::Literal::i64_suffixed(i as i64),
                    IntegerSuffix::Isize => proc_macro::Literal::isize_suffixed(i as isize),
                }
            }
            Literal::Integer(i, None) => proc_macro::Literal::usize_unsuffixed(i),
            Literal::Float(f, Some(suffix)) => {
                match suffix {
                    FloatSuffix::F32 => proc_macro::Literal::f32_suffixed(f as f32),
                    FloatSuffix::F64 => proc_macro::Literal::f64_suffixed(f),
                }
            }
            Literal::Float(f, None) => proc_macro::Literal::f64_unsuffixed(f),
            Literal::Str(s) => proc_macro::Literal::string(&s),
            Literal::ByteStr(bs) => proc_macro::Literal::byte_string(&bs),
            Literal::CStr(cstr) => {
                let s = cstr.to_str().expect("Failed to convert CString to str");
                proc_macro::Literal::string(s)
            }
        }
    }
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::Char(c) => write!(f, "'{}'", c),
            Literal::Integer(i, Some(suffix)) => write!(f, "{}{}", i, suffix.as_str()),
            Literal::Integer(i, None) => write!(f, "{}", i),
            Literal::Float(fl, Some(suffix)) => write!(f, "{}{}", fl, suffix.as_str()),
            Literal::Float(fl, None) => write!(f, "{}", fl),
            Literal::Str(s) => write!(f, "\"{}\"", s),
            Literal::ByteStr(bs) => write!(f, "b\"{}\"", String::from_utf8_lossy(bs)),
            Literal::CStr(cstr) => write!(f, "c\"{}\"", cstr.to_str().expect("Failed to convert CString to str")),
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
        let mut group = proc_macro::Group::new(g.delimiter().into(), g.stream.into());
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
    pub const fn new(delimiter: Delimiter, stream: TokenStream) -> Self {Self {delimiter, stream}}

    pub const fn stream(&self) -> &TokenStream {&self.stream}
    pub fn take_stream(self) -> TokenStream {self.stream}
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
            proc_macro::Delimiter::Parenthesis => Delimiter::Parenthesis,
            proc_macro::Delimiter::Brace => Delimiter::Brace,
            proc_macro::Delimiter::Bracket => Delimiter::Bracket,
            proc_macro::Delimiter::None => Delimiter::None,
        }
    }
}

#[cfg(feature = "in_proc_macro")]
impl From<Delimiter> for proc_macro::Delimiter {
    fn from(d: Delimiter) -> Self {
        match d {
            Delimiter::Parenthesis => proc_macro::Delimiter::Parenthesis,
            Delimiter::Brace => proc_macro::Delimiter::Brace,
            Delimiter::Bracket => proc_macro::Delimiter::Bracket,
            Delimiter::None => proc_macro::Delimiter::None,
        }
    }
}
