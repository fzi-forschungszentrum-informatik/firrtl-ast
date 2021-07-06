//! Error types

use std::error::Error as Error;
use std::fmt;
use std::io;


/// Parsing error type
#[derive(Debug)]
pub enum ParseError {
    IO(io::Error),
    Other(String),
}

impl From<io::ErrorKind> for ParseError {
    fn from(err: io::ErrorKind) -> Self {
        Self::IO(err.into())
    }
}

impl From<io::Error> for ParseError {
    fn from(err: io::Error) -> Self {
        Self::IO(err)
    }
}

impl From<String> for ParseError {
    fn from(err: String) -> Self {
        Self::Other(err)
    }
}

impl Error for ParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::IO(err) => Some(err),
            _ => None,
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IO(_)         => fmt::Display::fmt("IO error", f),
            Self::Other(err)    => fmt::Display::fmt(err, f),
        }
    }
}

