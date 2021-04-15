//! Parsers for modules and related items

use nom::branch::alt;
use nom::combinator::{map, value};
use nom::sequence::tuple;

use crate::parsers::{IResult, identifier, kw, op, spaced};
use crate::types::Type;
use crate::types::parsers::r#type;


/// Parse the elements of a port
fn port<'i>(input: &str) -> IResult<(String, Type, super::Direction)> {
    map(
        tuple((direction, spaced(identifier), spaced(op(":")), spaced(r#type))),
        |(direction, name, _, r#type)| (name.to_string(), r#type, direction)
    )(input)
}


/// Parse a direction
pub fn direction(input: &str) -> IResult<super::Direction> {
    use super::Direction as D;

    alt((
        value(D::Input, kw("input")),
        value(D::Output, kw("output")),
    ))(input)
}

