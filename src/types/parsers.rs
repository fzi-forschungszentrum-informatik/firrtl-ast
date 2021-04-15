//! Parsers for types

use nom::branch::alt;
use nom::combinator::{map, opt, value};
use nom::sequence::{preceded, tuple};
use nom::multi::{fold_many0, separated_list1};

use crate::parsers::{IResult, decimal, identifier, kw, op, spaced};


/// Parse a ground type
pub fn ground_type(input: &str) -> IResult<super::GroundType> {
    use super::GroundType as G;

    let bitwidth = |i| opt(map(spaced(tuple((op("<"), decimal, op(">")))), |(_, w, _)| w))(i);
    let point_offset = |i| opt(spaced(map(tuple((op("<<"), decimal, op(">>"))), |(_, w, _)| w)))(i);

    alt((
        map(preceded(kw("UInt"), bitwidth), G::UInt),
        map(preceded(kw("SInt"), bitwidth), G::SInt),
        map(tuple((kw("Fixed"), bitwidth, point_offset)), |(_, w, o)| G::Fixed(w, o)),
        value(G::Clock, kw("Clock")),
        map(preceded(kw("Analog"), bitwidth), G::Analog),
    ))(input)
}


/// Parse a type
pub fn r#type(input: &str) -> IResult<super::Type> {
    use super::Type as T;

    let field = map(
        tuple((opt(kw("flip")), spaced(identifier), spaced(op(":")), spaced(r#type))),
        |(o, n, _, t)| (n.to_string(), t, o.map(|_| super::Orientation::Flipped).unwrap_or_default())
    );

    let (input, res) = alt((
        map(
            tuple((op("{"), separated_list1(spaced(op(",")), spaced(field)), spaced(op("}")))),
            |(_, v, _)| T::Bundle(v)
        ),
        map(ground_type, T::GroundType),
    ))(input)?;

    fold_many0(
        spaced(tuple((op("["), spaced(decimal), spaced(op("]"))))),
        res,
        |t, (_, w, _)| T::Vector(Box::new(t), w)
    )(input)
}
