//! Test related to expressions

use nom::combinator::all_consuming;
use nom::Finish;

use crate::tests::{Equivalence, Identifier};

use super::{Expression, parsers, primitive};


#[quickcheck]
fn parse_expr(original: Expression<Identifier>) -> Result<Equivalence<Expression<Identifier>>, String> {
    let s = original.to_string();
    let res = all_consuming(|i| parsers::expr(|s| Some(s.into()), i))(&s)
        .finish()
        .map(|(_, parsed)| Equivalence::of(original, parsed))
        .map_err(|e| e.to_string());
    res
}


#[quickcheck]
fn parse_primitive_op(
    original: primitive::Operation<Identifier>
) -> Result<Equivalence<primitive::Operation<Identifier>>, String> {
    let s = original.to_string();
    let res = all_consuming(|i| parsers::primitive_op(|s| Some(s.into()), i))(&s)
        .finish()
        .map(|(_, parsed)| Equivalence::of(original, parsed))
        .map_err(|e| e.to_string());
    res
}

