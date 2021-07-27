// Copyright (c) 2021 FZI Forschungszentrum Informatik
// SPDX-License-Identifier: Apache-2.0
//! Error types

use std::error::Error as Error;
use std::fmt;
use std::io;

use crate::parsers;


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


/// Convert a `nom::Err` into a `ParseError`
pub(crate) fn convert_error(input: &str, err: nom::Err<parsers::Error>) -> ParseError {
    use nom::error::convert_error;

    match err {
        nom::Err::Incomplete(_) => io::ErrorKind::UnexpectedEof.into(),
        nom::Err::Error(e) | nom::Err::Failure(e) => convert_error(input, e).into(),
    }
}

