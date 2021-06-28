//! Parsers for modules and related items

use std::sync::Arc;

use nom::branch::alt;
use nom::combinator::{iterator, map, value};
use nom::sequence::tuple;

use crate::parsers::{IResult, identifier, kw, le, op, spaced};
use crate::types::parsers::r#type;
use crate::indentation::Indentation;


/// Parse a Module
pub fn module<'i>(input: &'i str, indentation: &'_ mut Indentation) -> IResult<'i, super::Module> {
    let (input, name) = map(
        tuple((indentation.parser(), kw("module"), spaced(identifier), spaced(op(":")), le)),
        |(_, _, name, ..)| name.into()
    )(input)?;

    let mut indentation = indentation.sub();

    let mut ports = iterator(input, map(tuple((indentation.parser(), port, le)), |(_, p, ..)| Arc::new(p)));
    let res = super::Module::new(name, &mut ports, super::Kind::Regular);
    ports.finish().map(|(i, _)| (i, res))
}


/// Parse a module instance
pub fn instance<'i>(
    module: impl Fn(&str) -> Option<Arc<super::Module>>,
    input: &'i str,
) -> IResult<'i, super::Instance> {
    nom::combinator::map_opt(
        tuple((kw("inst"), spaced(identifier), spaced(kw("of")), spaced(identifier))),
        |(_, inst_name, _, mod_name)| module(mod_name).map(|m| super::Instance::new(inst_name, m)),
    )(input)
}


/// Parse the elements of a port
pub fn port<'i>(input: &str) -> IResult<super::Port> {
    map(
        tuple((direction, spaced(identifier), spaced(op(":")), spaced(r#type))),
        |(direction, name, _, r#type)| super::Port::new(name.to_string(), r#type, direction)
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

