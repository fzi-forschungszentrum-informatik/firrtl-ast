//! Parsers for Circuits

use std::sync::Arc;

use nom::combinator::{iterator, map};
use nom::sequence::tuple;

use crate::indentation::Indentation;
use crate::module::parsers::module;
use crate::parsers::{Error, IResult, identifier, kw, le, op, spaced};


/// Parse a Circuit
pub fn circuit(input: &str) -> IResult<super::Circuit> {
    use nom::error::{ErrorKind as EK, ParseError};

    let (input, top_name) = map(
        tuple((kw("circuit"), spaced(identifier), spaced(op(":")), le)),
        |(_, n, ..)| n
    )(input)?;

    let mut indent = Indentation::root().sub();
    let mut modules = iterator(input, map(|i| module(i, &mut indent), Arc::new));
    let res = super::ModuleConsumer::new(top_name, &mut modules)
        .into_circuit()
        .ok_or_else(|| nom::Err::Error(Error::from_error_kind(input, EK::MapOpt)))?;
    modules.finish().map(|(i, _)| (i, res))
}
