//! Test related to registers

use nom::combinator::all_consuming;
use nom::Finish;

use crate::tests::{Equivalence, Identifier};

use super::{Register, parsers};

#[quickcheck]
fn parse_expr(original: Register<Identifier>) -> Result<Equivalence<Register<Identifier>>, String> {
    let s = original.to_string();
    let res = all_consuming(|i| parsers::register(|s| Some(s.into()), i))(&s)
        .finish()
        .map(|(_, parsed)| Equivalence::of(original, parsed))
        .map_err(|e| e.to_string());
    res
}

