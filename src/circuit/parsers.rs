//! Parsers for Circuits

use nom::combinator::{iterator, map};
use nom::sequence::tuple;

use crate::indentation::Indentation;
use crate::info::parse as parse_info;
use crate::module::parsers::Modules;
use crate::parsers::{Error, IResult, identifier, kw, le, op, spaced};


/// Parse a Circuit
pub fn circuit(input: &str) -> IResult<super::Circuit> {
    use nom::error::{ErrorKind as EK, ParseError};

    let (input, (top_name, info)) = map(
        tuple((kw("circuit"), spaced(identifier), spaced(op(":")), parse_info, le)),
        |(_, n, _, i, ..)| (n, i)
    )(input)?;

    let mut mod_parser = Modules::default();
    let mut indent = Indentation::root().sub();
    let mut modules = iterator(input, |i| mod_parser.parse_module(i, &mut indent));
    let res = super::ModuleConsumer::new(top_name, info, &mut modules)
        .into_circuit()
        .ok_or_else(|| nom::Err::Error(Error::from_error_kind(input, EK::MapOpt)))?;
    modules.finish().map(|(i, _)| (i, res))
}
