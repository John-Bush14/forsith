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
            return Err($crate::error::Error::msg(&format!($($arg)*)));
        }
    };
}
#[macro_export]
macro_rules! errmsg {
    ($($arg:tt)*) => {
        $crate::error::Error::msg(&format!($($arg)*))
    };
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    error: ErrorKind,
    context: Vec<Box<str>>,
}

#[derive(Debug)]
enum ErrorKind {
    Message(Box<str>),
    Error(Box<dyn std::error::Error>),
}
impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorKind::Message(msg) => write!(f, "{msg}"),
            ErrorKind::Error(err) => write!(f, "{err}"),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error)?;
        for (i, ctx) in self.context.iter().enumerate() {
            write!(f, "\n {i}. {ctx}")?;
        }
        Ok(())
    }
}

impl<T: std::error::Error + 'static> From<T> for Error {
    fn from(err: T) -> Self {
        Self {
            error: ErrorKind::Error(Box::new(err)),
            context: Vec::new(),
        }
    }
}

pub trait ResultContext<T> {
    fn with_context<R: Into<Box<str>>, F: FnOnce() -> R>(self, context: F) -> Self;
}
impl<T> ResultContext<T> for std::result::Result<T, Error> {
    fn with_context<R: Into<Box<str>>, F: FnOnce() -> R>(mut self, context: F) -> Self {
        if let Err(ref mut err) = self {
            err.context.push(context().into());
        }
        self
    }
}

impl Error {
    pub fn with_context(mut self, context: &str) -> Self {
        self.context.push(context.into());
        self
    }

    pub fn msg(msg: &str) -> Self {
        Self {
            error: ErrorKind::Message(msg.into()),
            context: Vec::new(),
        }
    }
}
