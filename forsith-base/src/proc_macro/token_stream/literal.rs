#[cfg(feature = "in_proc_macro")]
extern crate proc_macro;
use alloc::ffi::CString;
use crate::proc_macro::{TokenStream, TokenTree};

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
                let value =
                    l.to_string()[..l.to_string().len() - suffix.as_str().len()].to_string();
                if let Ok(i) = value.parse::<usize>() {
                    return Self::Integer(i, Some(*suffix));
                }
            }
        }

        for suffix in FloatSuffix::variants() {
            if l.to_string().ends_with(suffix.as_str()) {
                let value =
                    l.to_string()[..l.to_string().len() - suffix.as_str().len()].to_string();
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
            Literal::Integer(i, Some(suffix)) => match suffix {
                IntegerSuffix::U8 => Self::u8_suffixed(
                    i.try_into()
                        .expect("Literal with suffix u8 must be valid u8"),
                ),
                IntegerSuffix::U16 => Self::u16_suffixed(
                    i.try_into()
                        .expect("Literal with suffix u16 must be valid u16"),
                ),
                IntegerSuffix::U32 => Self::u32_suffixed(
                    i.try_into()
                        .expect("Literal with suffix u32 must be valid u32"),
                ),
                IntegerSuffix::U64 => Self::u64_suffixed(i as u64),
                IntegerSuffix::Usize => Self::usize_suffixed(i),
                IntegerSuffix::I8 => Self::i8_suffixed(
                    i.try_into()
                        .expect("Literal with suffix i8 must be valid i8"),
                ),
                IntegerSuffix::I16 => Self::i16_suffixed(
                    i.try_into()
                        .expect("Literal with suffix i16 must be valid i16"),
                ),
                IntegerSuffix::I32 => Self::i32_suffixed(
                    i.try_into()
                        .expect("Literal with suffix i32 must be valid i32"),
                ),
                IntegerSuffix::I64 => Self::i64_suffixed(
                    i.try_into()
                        .expect("Literal with suffix i64 must be valid i64"),
                ),
                IntegerSuffix::Isize => Self::isize_suffixed(
                    i.try_into()
                        .expect("Literal with suffix isize must be valid isize"),
                ),
            },
            Literal::Integer(i, None) => Self::usize_unsuffixed(i),
            Literal::Float(f, Some(suffix)) =>
            {
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

impl From<&str> for Literal {
    fn from(s: &str) -> Self {
        Self::Str(s.to_string())
    }
}

impl From<usize> for Literal {
    fn from(i: usize) -> Self {
        Self::Integer(i, None)
    }
}

impl core::fmt::Display for Literal {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Char(c) => write!(f, "'{c}'"),
            Self::Integer(i, Some(suffix)) => write!(f, "{}{}", i, suffix.as_str()),
            Self::Integer(i, None) => write!(f, "{i}"),
            Self::Float(fl, Some(suffix)) => write!(f, "{}{}", fl, suffix.as_str()),
            Self::Float(fl, None) => write!(f, "{fl}"),
            Self::Str(s) => write!(f, "\"{s}\""),
            Self::ByteStr(bs) => write!(f, "b\"{}\"", String::from_utf8_lossy(bs)),
            Self::CStr(cstr) => write!(
                f,
                "c\"{}\"",
                cstr.to_str().expect("Failed to convert CString to str")
            ),
        }
    }
}

impl Extend<Literal> for TokenStream {
    fn extend<T: IntoIterator<Item = Literal>>(&mut self, iter: T) {
        self.0.extend(iter.into_iter().map(TokenTree::Literal));
    }
}
