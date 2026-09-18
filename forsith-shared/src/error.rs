#[allow(unused_imports)]
use crate::alloc::string::ToString;
use crate::alloc::{boxed::Box, vec::Vec};

#[macro_export]
macro_rules! bail {
    ($($arg:tt)*) => {
        return Err(errmsg!($($arg)*))
    };
}
#[macro_export]
macro_rules! ensure {
    ($cond:expr, $($arg:tt)*) => {
        if !$cond {
            return Err($crate::error::Error::msg(&$crate::alloc::format!($($arg)*)));
        }
    };
}
#[macro_export]
macro_rules! errmsg {
    ($($arg:tt)*) => {
        $crate::error::Error::msg(&$crate::alloc::format!($($arg)*))
    };
}

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    error: ErrorKind,
    context: Vec<Box<str>>,
}

#[derive(Debug)]
enum ErrorKind {
    Message(Box<str>),
    Error(Box<dyn core::error::Error>),
}
impl core::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Message(msg) => write!(f, "{msg}"),
            Self::Error(err) => write!(f, "{err}"),
        }
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.error)?;
        for (i, ctx) in self.context.iter().enumerate() {
            write!(f, "\n {i}. {ctx}")?;
        }
        Ok(())
    }
}

impl<T: core::error::Error + 'static> From<T> for Error {
    fn from(err: T) -> Self {
        Self {
            error: ErrorKind::Error(Box::new(err)),
            context: Vec::new(),
        }
    }
}

pub trait ResultContext<T> {
    #[must_use]
    fn with_context<R: Into<Box<str>>, F: FnOnce() -> R>(self, context: F) -> Self;
}
impl<T> ResultContext<T> for core::result::Result<T, Error> {
    fn with_context<R: Into<Box<str>>, F: FnOnce() -> R>(mut self, context: F) -> Self {
        if let Err(ref mut err) = self {
            err.context.push(context().into());
        }
        self
    }
}

impl Error {
    #[must_use]
    pub fn with_context(mut self, context: &str) -> Self {
        self.context.push(context.into());
        self
    }

    #[must_use]
    pub fn msg(msg: &str) -> Self {
        Self {
            error: ErrorKind::Message(msg.into()),
            context: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_msg_creates_message_error() {
        let err = Error::msg("bad input");
        assert_eq!(err.to_string(), "bad input");
    }

    #[test]
    fn error_with_context_adds_context() {
        let err = Error::msg("bad input").with_context("during parsing");
        assert_eq!(err.to_string(), "bad input\n 0. during parsing");
    }

    #[test]
    fn result_context_accumulates_multiple_contexts() {
        let err = Err::<(), _>(Error::msg("bad input"))
            .with_context(|| "first")
            .with_context(|| "second")
            .unwrap_err();

        assert_eq!(err.to_string(), "bad input\n 0. first\n 1. second");
    }

    #[test]
    fn ensure_macro_returns_error_for_false_condition() {
        let result: Result<()> = (|| {
            ensure!(false, "boom");
            Ok(())
        })();

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "boom");
    }

    #[test]
    fn bail_macro_returns_error_and_can_wrap_format_strings() {
        let result: Result<()> = (|| -> Result<()> {
            bail!("unexpected value: {}", 42);
        })();

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "unexpected value: 42");
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_error_from_std_error() {
        (|| -> Result<()> {
            core::result::Result::Err(std::io::Error::other("test"))?;

            Ok(())
        })()
        .unwrap_err();
    }
}
