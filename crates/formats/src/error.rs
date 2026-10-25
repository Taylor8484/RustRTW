use std::fmt;
use std::io;
use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    /// An I/O error while reading `path`.
    Io { path: PathBuf, source: io::Error },
    /// The data ended before a value starting at `offset` could be read.
    UnexpectedEof { offset: usize, wanted: usize },
    /// The data was readable but not valid for the format.
    Invalid(String),
    /// The requested file is in neither the pack nor the game folder.
    NotFound(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io { path, source } => write!(f, "{}: {source}", path.display()),
            Error::UnexpectedEof { offset, wanted } => {
                write!(
                    f,
                    "unexpected end of data reading {wanted} bytes at offset {offset}"
                )
            }
            Error::Invalid(msg) => write!(f, "invalid data: {msg}"),
            Error::NotFound(name) => write!(f, "file not found: {name}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

pub(crate) fn io_error(path: impl Into<PathBuf>) -> impl FnOnce(io::Error) -> Error {
    let path = path.into();
    move |source| Error::Io { path, source }
}
