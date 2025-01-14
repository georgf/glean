use std::fmt::{self, Display};
use std::io;
use std::result;

use failure::{self, Backtrace, Context, Fail};

use ffi_support::{handle_map::HandleError, ExternError};

use rkv::error::StoreError;

/// A specialized [`Result`] type for this crate's operations.
///
/// This is generally used to avoid writing out [Error] directly and
/// is otherwise a direct mapping to [`Result`].
///
/// [`Result`]: https://doc.rust-lang.org/stable/std/result/enum.Result.html
/// [`Error`]: std.struct.Error.html
pub type Result<T> = result::Result<T, Error>;

/// A list enumerating the categories of errors in this crate.
///
/// This list is intended to grow over time and it is not recommended to
/// exhaustively match against it.
///
/// It is used with the [`Error`] struct.
///
/// [`Error`]: std.struct.Error.html
#[derive(Debug, Fail)]
pub enum ErrorKind {
    /// Lifetime conversion failed
    #[fail(display = "Lifetime conversion from {} failed", _0)]
    Lifetime(i32),

    /// FFI-Support error
    #[fail(display = "Invalid handle")]
    Handle(HandleError),

    /// IO error
    #[fail(display = "An I/O error occurred.")]
    IoError(io::Error),

    /// IO error
    #[fail(display = "An Rkv error occurred.")]
    Rkv(StoreError),

    /// JSON error
    #[fail(display = "A JSON error occurred.")]
    Json(serde_json::error::Error),

    /// TimeUnit conversion failed
    #[fail(display = "TimeUnit conversion from {} failed", _0)]
    TimeUnit(i32),
}

/// A specialized [`Error`] type for this crate's operations.
///
/// [`Error`]: https://doc.rust-lang.org/stable/std/error/trait.Error.html
#[derive(Debug)]
pub struct Error {
    inner: Context<ErrorKind>,
}

impl Error {
    /// Access the [`ErrorKind`] member.
    ///
    /// [`ErrorKind`]: enum.ErrorKind.html
    pub fn kind(&self) -> &ErrorKind {
        &*self.inner.get_context()
    }
}

impl Fail for Error {
    fn cause(&self) -> Option<&dyn Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error {
        let inner = Context::new(kind);
        Error { inner }
    }
}

impl From<Context<ErrorKind>> for Error {
    fn from(inner: Context<ErrorKind>) -> Error {
        Error { inner }
    }
}

impl From<HandleError> for Error {
    fn from(error: HandleError) -> Error {
        Error {
            inner: Context::new(ErrorKind::Handle(error)),
        }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Error {
        Error {
            inner: Context::new(ErrorKind::IoError(error)),
        }
    }
}

impl From<StoreError> for Error {
    fn from(error: StoreError) -> Error {
        Error {
            inner: Context::new(ErrorKind::Rkv(error)),
        }
    }
}

impl From<Error> for ExternError {
    fn from(error: Error) -> ExternError {
        ffi_support::ExternError::new_error(ffi_support::ErrorCode::new(42), format!("{}", error))
    }
}

impl From<serde_json::error::Error> for Error {
    fn from(error: serde_json::error::Error) -> Error {
        Error {
            inner: Context::new(ErrorKind::Json(error)),
        }
    }
}
