//! Parsers for modules and related items

use nom::branch::alt;
use nom::combinator::value;

use crate::parsers::{IResult, kw};


/// Parse a direction
pub fn direction(input: &str) -> IResult<super::Direction> {
    use super::Direction as D;

    alt((
        value(D::Input, kw("input")),
        value(D::Output, kw("output")),
    ))(input)
}

