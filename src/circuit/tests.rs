//! Tests related to circuits

use nom::Finish;
use nom::combinator::all_consuming;

use crate::tests::Equivalence;

use super::{Circuit, parsers};


#[quickcheck]
fn parse_circuit(original: Circuit) -> Result<Equivalence<Circuit>, String> {
    let s = original.to_string();
    let res = all_consuming(parsers::circuit)(&s)
        .finish()
        .map(|(_, parsed)| Equivalence::of(original, parsed))
        .map_err(|e| e.to_string());
    res
}
