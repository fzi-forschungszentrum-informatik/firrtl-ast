//! Parsers for Circuits

use std::sync::Arc;

use nom::character::complete::line_ending;
use nom::combinator::{map, map_opt};
use nom::multi::many1;
use nom::sequence::tuple;

use crate::indentation::Indentation;
use crate::module::parsers::module;
use crate::parsers::{IResult, identifier, kw, op, spaced};


/// Parse a Circuit
pub fn circuit(input: &str) -> IResult<super::Circuit> {
    let mut indent = Indentation::root().sub();
    let res = map_opt(
        tuple((
            kw("circuit"),
            spaced(identifier),
            spaced(op(":")),
            line_ending,
            many1(map(|i| module(i, &mut indent), Arc::new))
        )),
        |(_, top_name, .., modules)| {
            let top = modules.iter().find(|m| m.name() == top_name)?.clone();
            Some(super::Circuit::new(top, modules))
        }
    )(input);
    res
}
