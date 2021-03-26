//! Parsers for types

use nom::branch::alt;
use nom::combinator::{map, opt, value};
use nom::sequence::{preceded, tuple};

use crate::parsers::{IResult, decimal, kw, op};


/// Parse a ground type
pub fn ground_type(input: &str) -> IResult<super::GroundType> {
    use super::GroundType as G;

    let bitwidth = |i| opt(map(tuple((op("<"), decimal, op(">"))), |(_, w, _)| w))(i);
    let point_offset = |i| opt(map(tuple((op("<<"), decimal, op(">>"))), |(_, w, _)| w))(i);

    alt((
        map(preceded(kw("UInt"), bitwidth), G::UInt),
        map(preceded(kw("SInt"), bitwidth), G::SInt),
        map(tuple((kw("Fixed"), bitwidth, point_offset)), |(_, w, o)| G::Fixed(w, o)),
        value(G::Clock, kw("Clock")),
        map(preceded(kw("Analog"), bitwidth), G::Analog),
    ))(input)
}

