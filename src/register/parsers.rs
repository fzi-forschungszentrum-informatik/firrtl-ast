//! Parsers for types

use nom::combinator::{map, opt};
use nom::sequence::tuple;

use crate::expr::Reference;
use crate::expr::parsers::expr;
use crate::parsers::{IResult, comma, identifier, kw, lp, op, rp, spaced};
use crate::types::parsers::r#type;


/// Parse a register definition
pub fn register<'i, R: Reference + Clone>(
    reference: impl Fn(&str) -> Option<R> + Copy,
    input: &'i str
) -> IResult<'i, super::Register<R>> {
    use nom::Parser;

    let expr = |i| spaced(|i| expr(reference, i)).parse(i);

    let reset = map(
        tuple((lp, spaced(kw("reset")), spaced(op("=>")), lp, &expr, comma, &expr, rp, rp)),
        |(.., sig, _, val, _, _)| (sig, val)
    );

    let res = map(
        tuple((
            kw("reg"),
            spaced(identifier),
            spaced(op(":")),
            spaced(r#type),
            comma,
            &expr,
            opt(spaced(map(tuple((kw("with"), spaced(op(":")), spaced(reset))), |(.., r)| r)))
        )),
        |(_, name, _, r#type, _, clock, reset)| super::Register::new(name, r#type, clock)
            .with_optional_reset(reset)
    )(input);
    res
}

