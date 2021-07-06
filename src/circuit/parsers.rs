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

    let (input, (top_name, info)) = head(input)?;

    let mut mod_parser = Modules::default();
    let mut indent = Indentation::root().sub();
    let mut modules = iterator(input, |i| mod_parser.parse_module(i, &mut indent));
    let res = super::ModuleConsumer::new(top_name, info, &mut modules)
        .into_circuit()
        .ok_or_else(|| nom::Err::Error(Error::from_error_kind(input, EK::MapOpt)))?;
    modules.finish().map(|(i, _)| (i, res))
}


/// Parse a circuit's "head"
///
/// This parser parses the first line of a circuit's definition, which contains
/// the top module's name and an optional info attribute. On success, the parser
/// will yield a tuple with the top module's name as its first element and the
/// info as its second.
pub fn head<'i>(input: &'i str) -> IResult<'i, (&'i str, Option<String>)> {
    map(
        tuple((kw("circuit"), spaced(identifier), spaced(op(":")), parse_info, le)),
        |(_, n, _, i, ..)| (n, i)
    )(input)
}

