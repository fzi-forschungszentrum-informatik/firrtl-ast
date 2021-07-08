//! Parsers for expressions

use std::num::ParseIntError;
use std::sync::Arc;

use nom::branch::alt;
use nom::combinator::{map, map_opt, value};
use nom::sequence::{preceded, terminated, tuple};
use nom::multi::fold_many0;

use crate::parsers::{IResult, comma, decimal, identifier, kw, lp, op, rp, spaced};
use crate::types;


pub fn expr<'i, R: super::Reference + Clone>(
    reference: impl Fn(&str) -> Option<R> + Copy,
    input: &'i str
) -> IResult<'i, super::Expression<R>> {
    use types::parsers::{bitwidth, field_name};

    use super::Expression as E;

    let sub = |i| map(spaced(|i| expr(reference, i)), Arc::new)(i);

    let (input, res) = alt((
        map(
            tuple((kw("UInt"), spaced(bitwidth), lp, spaced(num_lit), rp)),
            |(_, width, _, value, _)| {
                let width = width
                    .or_else(|| (0..u16::MAX).find(|i| value >> i == 0))
                    .expect("Could not determine appropriate width");
                E::UIntLiteral{value, width}
            }
        ),
        map(
            tuple((kw("SInt"), spaced(bitwidth), lp, spaced(num_lit), rp)),
            |(_, width, _, value, _)| {
                let width = width
                    .or_else(|| (1..u16::MAX).find(|i| value >> (i - 1) == 0 || value >> i == -1))
                    .expect("Could not determine appropriate width");
                E::SIntLiteral{value, width}
            }
        ),
        map(
            tuple((kw("mux"), lp, &sub, comma, &sub, comma, &sub, rp)),
            |(_, _, sel, _, a, _, b, _)| E::Mux{sel, a, b}
        ),
        map(
            tuple((kw("validif"), lp, &sub, comma, &sub, rp)),
            |(_, _, sel, _, value, _)| E::ValidIf{sel, value}
        ),
        map(|i| primitive_op(reference, i), E::PrimitiveOp),
        map_opt(identifier, |name| reference(name).map(E::Reference)),
    ))(input)?;

    /// Utility enum for parsing subscripts
    enum Subscript<R: super::Reference> {Field(Arc<str>), Index(u16), Access(Arc<super::Expression<R>>)}

    fold_many0(
        spaced(alt((
            map(preceded(op("."), spaced(field_name)), |i| Subscript::Field(Arc::from(i))),
            map(tuple((op("["), spaced(decimal), spaced(op("]")))), |(_, i, _)| Subscript::Index(i)),
            map(tuple((op("["), sub, spaced(op("]")))), |(_, i, _)| Subscript::Access(i)),
        ))),
        res,
        |e, s| match s {
            Subscript::Field(index)  => E::SubField{base: Arc::new(e), index},
            Subscript::Index(index)  => E::SubIndex{base: Arc::new(e), index},
            Subscript::Access(index) => E::SubAccess{base: Arc::new(e), index},
        }
    )(input)
}


/// Parse a primitive operation
pub fn primitive_op<'i, R: super::Reference + Clone>(
    reference: impl Fn(&str) -> Option<R> + Copy,
    input: &'i str
) -> IResult<'i, super::primitive::Operation<R>> {
    use nom::error::ParseError;

    use types::{GroundType as GT, ResetKind as RK};

    use super::primitive::Operation as PO;

    let sub = |i| map(spaced(|i| expr(reference, i)), Arc::new)(i);

    let (input, op) = terminated(identifier, lp)(input)?;
    let (input, op) = match op {
        "add"           => map(tuple((&sub, comma, &sub)), |(l, _, r)| PO::Add(l, r))(input)?,
        "sub"           => map(tuple((&sub, comma, &sub)), |(l, _, r)| PO::Sub(l, r))(input)?,
        "mul"           => map(tuple((&sub, comma, &sub)), |(l, _, r)| PO::Mul(l, r))(input)?,
        "div"           => map(tuple((&sub, comma, &sub)), |(l, _, r)| PO::Div(l, r))(input)?,
        "rem"           => map(tuple((&sub, comma, &sub)), |(l, _, r)| PO::Rem(l, r))(input)?,
        "lt"            => map(tuple((&sub, comma, &sub)), |(l, _, r)| PO::Lt(l, r))(input)?,
        "leq"           => map(tuple((&sub, comma, &sub)), |(l, _, r)| PO::LEq(l, r))(input)?,
        "gt"            => map(tuple((&sub, comma, &sub)), |(l, _, r)| PO::Gt(l, r))(input)?,
        "geq"           => map(tuple((&sub, comma, &sub)), |(l, _, r)| PO::GEq(l, r))(input)?,
        "eq"            => map(tuple((&sub, comma, &sub)), |(l, _, r)| PO::Eq(l, r))(input)?,
        "neq"           => map(tuple((&sub, comma, &sub)), |(l, _, r)| PO::NEq(l, r))(input)?,
        "pad"           => map(tuple((&sub, comma, spaced(decimal))), |(e, _, b)| PO::Pad(e, b))(input)?,
        "asUInt"        => map(&sub, |e| PO::Cast(e, GT::UInt(None)))(input)?,
        "asSInt"        => map(&sub, |e| PO::Cast(e, GT::SInt(None)))(input)?,
        "asFixed"       => map(
            tuple((&sub, comma, spaced(decimal))),
            |(e, _, p)| PO::Cast(e, GT::Fixed(None, Some(p)))
        )(input)?,
        "asClock"       => map(&sub, |e| PO::Cast(e, GT::Clock))(input)?,
        "asAsyncReset"  => map(&sub, |e| PO::Cast(e, GT::Reset(RK::Async)))(input)?,
        "shl"           => map(tuple((&sub, comma, spaced(decimal))), |(e, _, b)| PO::Shl(e, b))(input)?,
        "shr"           => map(tuple((&sub, comma, spaced(decimal))), |(e, _, b)| PO::Shr(e, b))(input)?,
        "dshl"          => map(tuple((&sub, comma, &sub)), |(e, _, b)| PO::DShl(e, b))(input)?,
        "dshr"          => map(tuple((&sub, comma, &sub)), |(e, _, b)| PO::DShr(e, b))(input)?,
        "cvt"           => map(&sub, PO::Cvt)(input)?,
        "neg"           => map(&sub, PO::Neg)(input)?,
        "not"           => map(&sub, PO::Not)(input)?,
        "and"           => map(tuple((&sub, comma, &sub)), |(l, _, r)| PO::And(l, r))(input)?,
        "or"            => map(tuple((&sub, comma, &sub)), |(l, _, r)| PO::Or(l, r))(input)?,
        "xor"           => map(tuple((&sub, comma, &sub)), |(l, _, r)| PO::Xor(l, r))(input)?,
        "andr"          => map(&sub, PO::AndReduce)(input)?,
        "orr"           => map(&sub, PO::OrReduce)(input)?,
        "xorr"          => map(&sub, PO::XorReduce)(input)?,
        "cat"           => map(tuple((&sub, comma, &sub)), |(l, _, r)| PO::Cat(l, r))(input)?,
        "bits"          => map(
            tuple((&sub, comma, spaced(decimal), comma, spaced(decimal))),
            |(e, _, l, _, h)| PO::Bits(e, Some(l), Some(h))
        )(input)?,
        "head"          => map(
            tuple((&sub, comma, spaced(decimal))),
            |(e, _, h)| PO::Bits(e, None, Some(h))
        )(input)?,
        "tail"          => map(
            tuple((&sub, comma, spaced(decimal))),
            |(e, _, l)| PO::Bits(e, Some(l), None)
        )(input)?,
        "incp"          => map(
            tuple((&sub, comma, spaced(decimal))),
            |(e, _, b)| PO::IncPrecision(e, b)
        )(input)?,
        "decp"          => map(
            tuple((&sub, comma, spaced(decimal))),
            |(e, _, b)| PO::DecPrecision(e, b)
        )(input)?,
        "setp"          => map(
            tuple((&sub, comma, spaced(decimal))),
            |(e, _, b)| PO::SetPrecision(e, b)
        )(input)?,
        _               => return Err(
            nom::Err::Error(crate::parsers::Error::from_error_kind(input, nom::error::ErrorKind::Tag))
        ),
    };

    value(op, rp)(input)
}


/// Parse FIRRTL's weird stringified number literal format
///
/// This parser yields the value and radix.
fn num_lit<T: FromStrRadix + std::str::FromStr>(input: &str) -> IResult<T> {
    use nom::character::complete::{alphanumeric1, char as chr};
    use nom::combinator::{map_res, recognize, opt};

    alt((
        decimal,
        map_res(
            tuple((
                chr('"'),
                alt((value(2, chr('b')), value(8, chr('o')), value(16, chr('h')))),
                recognize(preceded(opt(alt((chr('+'), chr('-')))), alphanumeric1)),
                chr('"'),
            )),
            |(_, radix, value, _)| FromStrRadix::from_str_radix(value, radix)
        )
    ))(input)
}


/// Helper trait for generalizing from_str_radix
trait FromStrRadix: Sized {
    fn from_str_radix(value: &str, radix: u32) -> Result<Self, ParseIntError>;
}

impl FromStrRadix for u128 {
    fn from_str_radix(value: &str, radix: u32) -> Result<Self, ParseIntError> {
        u128::from_str_radix(value, radix)
    }
}

impl FromStrRadix for i128 {
    fn from_str_radix(value: &str, radix: u32) -> Result<Self, ParseIntError> {
        i128::from_str_radix(value, radix)
    }
}

