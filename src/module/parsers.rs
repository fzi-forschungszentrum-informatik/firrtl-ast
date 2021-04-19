//! Parsers for modules and related items

use nom::branch::alt;
use nom::character::complete::line_ending;
use nom::combinator::{iterator, map, value};
use nom::sequence::tuple;

use crate::parsers::{IResult, identifier, kw, op, spaced};
use crate::types::Type;
use crate::types::parsers::r#type;
use crate::indentation::Indentation;


/// Parse a Module
pub fn module<'i>(input: &'i str, indentation: &'_ mut Indentation) -> IResult<'i, super::Module> {
    let (input, name) = map(
        tuple((indentation.parser(), kw("module"), spaced(identifier), spaced(op(":")), line_ending)),
        |(_, _, name, ..)| name.into()
    )(input)?;

    let mut indentation = indentation.sub();

    let mut ports = iterator(input, map(tuple((indentation.parser(), port, line_ending)), |(_, p, ..)| p));
    let res = super::Module::new(name, &mut ports);
    ports.finish().map(|(i, _)| (i, res))
}


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

