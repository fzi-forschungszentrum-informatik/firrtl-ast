//! Tests related to modules

use nom::Finish;
use nom::combinator::all_consuming;

use crate::tests::Equivalence;

use super::{Direction, parsers};


#[quickcheck]
fn parse_direction(original: Direction) -> Result<Equivalence<Direction>, String> {
    let s = original.to_string();
    let res = all_consuming(parsers::direction)(&s)
        .finish()
        .map(|(_, parsed)| Equivalence::of(original, parsed))
        .map_err(|e| e.to_string());
    res
}

