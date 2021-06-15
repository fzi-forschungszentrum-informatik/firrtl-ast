//! Parsers for types

use nom::combinator::{map, opt};
use nom::sequence::tuple;

use crate::expr::Reference;
use crate::expr::parsers::expr;
use crate::parsers::{IResult, identifier, kw, op, spaced};
use crate::types::parsers::r#type;


/// Parse a register definition
pub fn register<'i, R: Reference + Clone>(
    reference: impl Fn(&str) -> Option<R> + Copy,
    input: &'i str
) -> IResult<'i, super::Register<R>> {
    let reset = map(
        tuple((
            spaced(op("(")),
            spaced(kw("reset")),
            spaced(op("=>")),
            spaced(op("(")),
            spaced(|i| expr(reference, i)),
            spaced(op(",")),
            spaced(|i| expr(reference, i)),
            spaced(op(")")),
            spaced(op(")")),
        )),
        |(.., sig, _, val, _, _)| (sig, val)
    );

    map(
        tuple((
            kw("reg"),
            spaced(identifier),
            spaced(op(":")),
            spaced(r#type),
            spaced(op(",")),
            spaced(|i| expr(reference, i)),
            opt(spaced(map(tuple((kw("with"), spaced(op(":")), spaced(reset))), |(.., r)| r)))
        )),
        |(_, name, _, r#type, _, clock, reset)| super::Register::new(name, r#type, clock)
            .with_optional_reset(reset)
    )(input)
}

