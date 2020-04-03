use std::{fmt, io, result};

use std::error::Error;
use std::fmt::{Display, Formatter};
use tokio::task::JoinError;

pub type Result<T> = result::Result<T, ErrorKind>;

#[derive(Debug)]
pub enum ErrorKind {
    /// Wrapper around a std io::Error
    IOError(io::Error),
    /// Error in decoding the packet
    DecodingError(DecodingErrorKind),
    /// Expected header but could not be read from buffer.
    CouldNotReadHeader(String),
}

impl Error for ErrorKind {}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::IOError(e) => write!(f, "An IO Error occurred. Reason: {:?}.", e),
            ErrorKind::DecodingError(e) => write!(
                f,
                "Something went wrong with parsing the header. Reason: {:}.",
                e
            ),
            ErrorKind::CouldNotReadHeader(header) => write!(
                f,
                "Expected {} header but could not be read from buffer.",
                header
            ),
        }
    }
}

/// Errors that could occur while parsing packet contents
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DecodingErrorKind {
    /// The [PacketType] could not be read
    PacketType,
}

impl Display for DecodingErrorKind {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            DecodingErrorKind::PacketType => write!(fmt, "The packet type could not be read."),
        }
    }
}

impl From<io::Error> for ErrorKind {
    fn from(inner: io::Error) -> ErrorKind {
        ErrorKind::IOError(inner)
    }
}

impl From<JoinError> for ErrorKind {
    fn from(inner: JoinError) -> ErrorKind {
        ErrorKind::IOError(io::Error::from(inner))
    }
}