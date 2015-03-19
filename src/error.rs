use std::io;
use std::error;
use std::fmt;

#[derive(PartialEq)]
pub enum Error {
    UnsupportedType,
    InvalidStr,
    MissingElements,
    KeyIsNotString,
    IoError(io::Error),
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::UnsupportedType => "unsupported type",
            Error::InvalidStr => "invalid string",
            Error::MissingElements => "no elements in value",
            Error::KeyIsNotString => "key is not a string",
            Error::IoError(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::IoError(ref err) => err.cause(),
            _ => None,
        }
    }
}

impl error::FromError<io::Error> for Error {
    fn from_error(err: io::Error) -> Error {
        Error::IoError(err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::IoError(ref err) => err.fmt(f),
            _ => error::Error::description(self).fmt(f),
        }
    }
}
