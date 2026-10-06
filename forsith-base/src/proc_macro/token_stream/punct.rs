use crate::proc_macro::{TokenStream, TokenTree};

#[cfg(feature = "in_proc_macro")]
extern crate proc_macro;

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
            char: p
                .as_char()
                .try_into()
                .expect("Failed to convert proc_macro::Punct to PunctChar"),
            joint: p.spacing() == proc_macro::Spacing::Joint,
        }
    }
}

#[cfg(feature = "in_proc_macro")]
impl From<Punct> for proc_macro::Punct {
    fn from(p: Punct) -> Self {
        let spacing = if p.joint {
            proc_macro::Spacing::Joint
        } else {
            proc_macro::Spacing::Alone
        };
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
    pub const fn new(char: PunctChar, joint: bool) -> Self {
        Self { char, joint }
    }

    #[must_use]
    pub const fn char(&self) -> PunctChar {
        self.char
    }

    #[must_use]
    pub const fn joint(&self) -> bool {
        self.joint
    }
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

impl core::fmt::Display for Punct {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.joint {
            write!(f, "{}", char::from(self.char))
        } else {
            write!(f, "{} ", char::from(self.char))
        }
    }
}
