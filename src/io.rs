//! `std::io` shim.
//!
//! Provides `Error`, `ErrorKind`, `Result`, `Read`, `Write`, `Seek` with
//! the same shapes as `std::io` but no OS-error integration. Downstream
//! crates that only need the traits for buffering / framing compile
//! unchanged. Crates that try to do real I/O via `std::io` will need to
//! be ported anyway because the kernel exposes its own I/O surface.

use alloc::boxed::Box;
use alloc::string::String;
use core::fmt;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    msg: Option<Box<str>>,
}

impl Error {
    pub fn new<E: Into<Box<dyn core::error::Error + Send + Sync>>>(kind: ErrorKind, _err: E) -> Self {
        Self { kind, msg: None }
    }
    pub fn other<E: Into<Box<dyn core::error::Error + Send + Sync>>>(error: E) -> Self {
        Self::new(ErrorKind::Other, error)
    }
    pub fn from_raw_os_error(_code: i32) -> Self {
        Self { kind: ErrorKind::Other, msg: None }
    }
    pub fn raw_os_error(&self) -> Option<i32> {
        None
    }
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }
    pub fn get_ref(&self) -> Option<&(dyn core::error::Error + Send + Sync + 'static)> {
        None
    }
    pub fn into_inner(self) -> Option<Box<dyn core::error::Error + Send + Sync>> {
        None
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.msg {
            Some(m) => f.write_str(m),
            None => write!(f, "{:?}", self.kind),
        }
    }
}

impl core::error::Error for Error {}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        Self { kind, msg: None }
    }
}

impl From<&str> for Error {
    fn from(s: &str) -> Self {
        Self { kind: ErrorKind::Other, msg: Some(String::from(s).into_boxed_str()) }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ErrorKind {
    NotFound,
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
    FilesystemQuotaExceeded,
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
    Other,
}

pub trait Read {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;

    fn read_to_end(&mut self, buf: &mut alloc::vec::Vec<u8>) -> Result<usize> {
        let mut total = 0usize;
        let mut tmp = [0u8; 4096];
        loop {
            match self.read(&mut tmp)? {
                0 => return Ok(total),
                n => {
                    buf.extend_from_slice(&tmp[..n]);
                    total += n;
                }
            }
        }
    }

    fn read_exact(&mut self, mut buf: &mut [u8]) -> Result<()> {
        while !buf.is_empty() {
            match self.read(buf)? {
                0 => return Err(Error::from(ErrorKind::UnexpectedEof)),
                n => {
                    let tmp = buf;
                    buf = &mut tmp[n..];
                }
            }
        }
        Ok(())
    }
}

pub trait Write {
    fn write(&mut self, buf: &[u8]) -> Result<usize>;
    fn flush(&mut self) -> Result<()>;

    fn write_all(&mut self, mut buf: &[u8]) -> Result<()> {
        while !buf.is_empty() {
            match self.write(buf)? {
                0 => return Err(Error::from(ErrorKind::WriteZero)),
                n => buf = &buf[n..],
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SeekFrom {
    Start(u64),
    End(i64),
    Current(i64),
}

pub trait Seek {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64>;
}
