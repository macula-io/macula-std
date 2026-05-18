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

/// Minimal `BufRead` shim, just enough for bytes' Reader wrapper. Real std
/// implements `read_until` and friends on top; we ship only `fill_buf` +
/// `consume` because nothing else is reached in the kernel build.
pub trait BufRead: Read {
    fn fill_buf(&mut self) -> Result<&[u8]>;
    fn consume(&mut self, amt: usize);
}

/// In-memory `Cursor<T>` like `std::io::Cursor`, generic over any `T:
/// AsRef<[u8]>`. Supports `Read`, `Seek`, and (when `T: AsMut<[u8]>`)
/// `Write`. Position tracks bytes consumed; reads short-read at EOF.
#[derive(Debug, Clone)]
pub struct Cursor<T> {
    inner: T,
    pos: u64,
}

impl<T> Cursor<T> {
    pub const fn new(inner: T) -> Self {
        Self { inner, pos: 0 }
    }
    pub fn into_inner(self) -> T {
        self.inner
    }
    pub fn get_ref(&self) -> &T {
        &self.inner
    }
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.inner
    }
    pub fn position(&self) -> u64 {
        self.pos
    }
    pub fn set_position(&mut self, pos: u64) {
        self.pos = pos;
    }
}

impl<T: AsRef<[u8]>> Read for Cursor<T> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let bytes = self.inner.as_ref();
        let start = (self.pos as usize).min(bytes.len());
        let remaining = &bytes[start..];
        let n = remaining.len().min(buf.len());
        buf[..n].copy_from_slice(&remaining[..n]);
        self.pos += n as u64;
        Ok(n)
    }
}

impl<T: AsRef<[u8]>> Seek for Cursor<T> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        let new_pos = match pos {
            SeekFrom::Start(p) => p as i128,
            SeekFrom::End(p) => self.inner.as_ref().len() as i128 + p as i128,
            SeekFrom::Current(p) => self.pos as i128 + p as i128,
        };
        if new_pos < 0 {
            return Err(Error::from(ErrorKind::InvalidInput));
        }
        self.pos = new_pos as u64;
        Ok(self.pos)
    }
}

impl<T: AsRef<[u8]> + AsMut<[u8]>> Write for Cursor<T> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let bytes = self.inner.as_mut();
        let start = (self.pos as usize).min(bytes.len());
        let remaining = &mut bytes[start..];
        let n = remaining.len().min(buf.len());
        remaining[..n].copy_from_slice(&buf[..n]);
        self.pos += n as u64;
        Ok(n)
    }
    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

/// `IoSlice<'a>` mirrors `std::io::IoSlice` for vectored-I/O APIs. Our
/// kernel does no real vectored I/O yet, but bytes / quinn-proto reference
/// the type in trait signatures and need it to exist + impl `Deref<[u8]>`.
#[derive(Copy, Clone, Debug)]
#[repr(transparent)]
pub struct IoSlice<'a>(&'a [u8]);

impl<'a> IoSlice<'a> {
    pub const fn new(buf: &'a [u8]) -> Self {
        Self(buf)
    }
    pub fn advance(&mut self, n: usize) {
        self.0 = &self.0[n..];
    }
    pub fn as_slice(&self) -> &[u8] {
        self.0
    }
}

impl<'a> core::ops::Deref for IoSlice<'a> {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        self.0
    }
}

/// `IoSliceMut<'a>` mirrors `std::io::IoSliceMut`. Same caveat: kernel does
/// no real vectored I/O yet, but the type must exist for trait signatures.
#[derive(Debug)]
#[repr(transparent)]
pub struct IoSliceMut<'a>(&'a mut [u8]);

impl<'a> IoSliceMut<'a> {
    pub fn new(buf: &'a mut [u8]) -> Self {
        Self(buf)
    }
}

impl<'a> core::ops::Deref for IoSliceMut<'a> {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        self.0
    }
}

impl<'a> core::ops::DerefMut for IoSliceMut<'a> {
    fn deref_mut(&mut self) -> &mut [u8] {
        self.0
    }
}

impl Write for alloc::vec::Vec<u8> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}
