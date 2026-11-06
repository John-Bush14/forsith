pub trait Read {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;

    fn min_buffer_size(&self) -> usize {
        0
    }

    fn remaining(&self) -> Option<usize> {
        None
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        match self.read(buf) {
            Ok(n) if n == buf.len() => Ok(()),
            Ok(n) => self.read_exact(&mut buf[n..]),
            Err(e) => Err(e),
        }
    }

    fn read_to_vec(&mut self) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        self.read_to_end(&mut buf)?;
        Ok(buf)
    }

    fn read_to_string(&mut self) -> Result<String> {
        let buf = self.read_to_vec()?;
        String::from_utf8(buf).map_err(Error::other)
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<usize> {
        if let Some(remaining) = self.remaining() {
            buf.reserve(remaining);
            return self
                .read_exact(unsafe { &mut buf.spare_capacity_mut().assume_init_mut()[..remaining] })
                .map(|()| remaining);
        }

        let mut total_read = 0;
        loop {
            if buf.capacity() - buf.len() < 100 {
                buf.reserve(1024);
            }
            match self.read(unsafe { buf.spare_capacity_mut().assume_init_mut() }) {
                Ok(0) => return Ok(total_read),
                Ok(n) => {
                    unsafe { buf.set_len(buf.len() + n) };
                    total_read += n;
                }
                Err(e) => return Err(e),
            }
        }
    }
}

#[cfg(feature = "std")]
impl<R: std::io::Read> Read for R {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.read(buf).map_err(Error::from)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        self.read_exact(buf).map_err(Error::from)
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<usize> {
        self.read_to_end(buf).map_err(Error::from)
    }
}

pub trait ReadBuf: Read {
    fn fill_buf(&mut self) -> Result<&[u8]>;
    fn consume(&mut self, amt: usize);
}

#[cfg(feature = "std")]
impl<R: std::io::BufRead> ReadBuf for R {
    fn fill_buf(&mut self) -> Result<&[u8]> {
        self.fill_buf().map_err(Error::from)
    }

    fn consume(&mut self, amt: usize) {
        self.consume(amt);
    }
}

pub enum SeekPosition {
    Start(u64),
    End(i64),
    Current(i64),
}

pub trait Seek {
    fn seek(&mut self, pos: SeekPosition) -> Result<u64>;

    fn seek_relative(&mut self, offset: i64) -> Result<u64> {
        self.seek(SeekPosition::Current(offset))
    }

    fn rewind(&mut self) -> Result<()> {
        self.seek(SeekPosition::Start(0))?;
        Ok(())
    }
}

#[cfg(feature = "std")]
impl<S: std::io::Seek> Seek for S {
    fn seek(&mut self, pos: SeekPosition) -> Result<u64> {
        let std_pos = match pos {
            SeekPosition::Start(offset) => std::io::SeekFrom::Start(offset),
            SeekPosition::End(offset) => std::io::SeekFrom::End(offset),
            SeekPosition::Current(offset) => std::io::SeekFrom::Current(offset),
        };
        self.seek(std_pos).map_err(Error::from)
    }
}

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    err: Box<dyn core::error::Error>,
}

impl Error {
    pub fn new<E>(kind: ErrorKind, err: E) -> Self
    where
        E: Into<Box<dyn core::error::Error + Send + Sync>>,
    {
        Self {
            kind,
            err: err.into(),
        }
    }

    pub fn other<E>(err: E) -> Self
    where
        E: Into<Box<dyn core::error::Error + Send + Sync>>,
    {
        Self::new(ErrorKind::Other, err)
    }

    #[must_use]
    pub const fn kind(&self) -> &ErrorKind {
        &self.kind
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{} ({:?})", self.err, self.kind)
    }
}

impl core::error::Error for Error {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        Some(self.err.as_ref())
    }
}

#[cfg(feature = "std")]
impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        let kind = err.kind().into();
        let error = err.into_inner().unwrap_or_else(|| "".into());

        Self::new(kind, error)
    }
}

#[cfg(feature = "std")]
impl From<std::io::ErrorKind> for ErrorKind {
    fn from(kind: std::io::ErrorKind) -> Self {
        match kind {
            std::io::ErrorKind::NotFound => Self::NotFound,
            std::io::ErrorKind::PermissionDenied => Self::PermissionDenied,
            std::io::ErrorKind::ConnectionRefused => Self::ConnectionRefused,
            std::io::ErrorKind::ConnectionReset => Self::ConnectionReset,
            std::io::ErrorKind::HostUnreachable => Self::HostUnreachable,
            std::io::ErrorKind::NetworkUnreachable => Self::NetworkUnreachable,
            std::io::ErrorKind::ConnectionAborted => Self::ConnectionAborted,
            std::io::ErrorKind::NotConnected => Self::NotConnected,
            std::io::ErrorKind::AddrInUse => Self::AddrInUse,
            std::io::ErrorKind::AddrNotAvailable => Self::AddrNotAvailable,
            std::io::ErrorKind::NetworkDown => Self::NetworkDown,
            std::io::ErrorKind::BrokenPipe => Self::BrokenPipe,
            std::io::ErrorKind::AlreadyExists => Self::AlreadyExists,
            std::io::ErrorKind::WouldBlock => Self::WouldBlock,
            std::io::ErrorKind::NotADirectory => Self::NotADirectory,
            std::io::ErrorKind::IsADirectory => Self::IsADirectory,
            std::io::ErrorKind::DirectoryNotEmpty => Self::DirectoryNotEmpty,
            std::io::ErrorKind::ReadOnlyFilesystem => Self::ReadOnlyFilesystem,
            std::io::ErrorKind::StaleNetworkFileHandle => Self::StaleNetworkFileHandle,
            std::io::ErrorKind::Other => Self::Other,
            std::io::ErrorKind::InvalidInput => Self::InvalidInput,
            std::io::ErrorKind::InvalidData => Self::InvalidData,
            std::io::ErrorKind::TimedOut => Self::TimedOut,
            std::io::ErrorKind::WriteZero => Self::WriteZero,
            std::io::ErrorKind::StorageFull => Self::StorageFull,
            std::io::ErrorKind::NotSeekable => Self::NotSeekable,
            std::io::ErrorKind::QuotaExceeded => Self::QuotaExceeded,
            std::io::ErrorKind::FileTooLarge => Self::FileTooLarge,
            std::io::ErrorKind::ResourceBusy => Self::ResourceBusy,
            std::io::ErrorKind::ExecutableFileBusy => Self::ExecutableFileBusy,
            std::io::ErrorKind::Deadlock => Self::Deadlock,
            std::io::ErrorKind::CrossesDevices => Self::CrossesDevices,
            std::io::ErrorKind::TooManyLinks => Self::TooManyLinks,
            std::io::ErrorKind::InvalidFilename => Self::InvalidFilename,
            std::io::ErrorKind::ArgumentListTooLong => Self::ArgumentListTooLong,
            std::io::ErrorKind::Interrupted => Self::Interrupted,
            std::io::ErrorKind::Unsupported => Self::Unsupported,
            std::io::ErrorKind::UnexpectedEof => Self::UnexpectedEof,
            std::io::ErrorKind::OutOfMemory => Self::OutOfMemory,
            _ => unimplemented!("unstable std::io::ErrorKind variant is not supported"),
        }
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum ErrorKind {
    NotFound,
    TinyDestinationBuffer,
    PermissionDenied,
    ConnectionRefused,
    ConnectionReset,
    HostUnreachable,
    NetworkUnreachable,
    ConnectionAborted,
    NotConnected,
    AddrInUse,
    AddrNotAvailable,
    NetworkDown,
    BrokenPipe,
    AlreadyExists,
    WouldBlock,
    NotADirectory,
    IsADirectory,
    DirectoryNotEmpty,
    ReadOnlyFilesystem,
    FilesystemLoop,
    StaleNetworkFileHandle,
    InvalidInput,
    InvalidData,
    TimedOut,
    WriteZero,
    StorageFull,
    NotSeekable,
    QuotaExceeded,
    FileTooLarge,
    ResourceBusy,
    ExecutableFileBusy,
    Deadlock,
    CrossesDevices,
    TooManyLinks,
    InvalidFilename,
    ArgumentListTooLong,
    Interrupted,
    Unsupported,
    UnexpectedEof,
    OutOfMemory,
    TooManyOpenFiles,
    InputOutputError,
    Other,
}
