//! Tests for parser utilities

use nom::character::complete::newline;
use nom::combinator::all_consuming;
use nom::sequence::terminated;
use nom::Finish;

use crate::tests::{Equivalence, Identifier};


#[quickcheck]
fn parse_identifier(original: Identifier) -> Result<Equivalence<Identifier>, String> {
    let s = format!("{}\n", original);
    let res = all_consuming(terminated(super::identifier, newline))(&s)
        .finish()
        .map(|(_, parsed)| Equivalence::of(original, parsed.into()))
        .map_err(|e| e.to_string());
    res
}

