//! Tests for parser utilities

use nom::combinator::all_consuming;
use nom::Finish;

use crate::tests::{Equivalence, Identifier};


#[quickcheck]
fn parse_identifier(original: Identifier) -> Result<Equivalence<Identifier>, String> {
    let s = original.to_string();
    let res = all_consuming(super::identifier)(&s)
        .finish()
        .map(|(_, parsed)| Equivalence::of(original, parsed.into()))
        .map_err(|e| e.to_string());
    res
}


#[quickcheck]
fn parse_decimal(original: i128) -> Result<Equivalence<i128>, String> {
    let s = original.to_string();
    let res = all_consuming(super::decimal)(&s)
        .finish()
        .map(|(_, parsed)| Equivalence::of(original, parsed))
        .map_err(|e| e.to_string());
    res
}

