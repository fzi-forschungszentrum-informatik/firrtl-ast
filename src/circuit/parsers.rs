//! Parsers for Circuits

use nom::combinator::map;
use nom::sequence::tuple;

use crate::error::{ParseError, convert_error};
use crate::info::parse as parse_info;
use crate::module::parsers::Modules;
use crate::parsers::{IResult, identifier, kw, le, op, spaced};


/// Parse a Circuit
pub fn circuit(input: &str) -> Result<super::Circuit, ParseError> {
    consumer(input)?.into_circuit()
}


/// Create a ModuleConsumer for the given input
///
/// The input is expected to contain a full circuit definition. The function
/// will return a `ModuleConsumer` which will construct a `Circuit` from that
/// input.
pub fn consumer(input: &str) -> Result<super::ModuleConsumer<Modules, ParseError>, ParseError> {
    let (input, (top_name, info)) = head(input).map_err(|e| convert_error(input, e))?;

    Ok(super::ModuleConsumer::new(top_name, info, Modules::new(input)))
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

