//! Tests related to memories

use nom::Finish;
use nom::combinator::all_consuming;

use crate::indentation::{DisplayIndented, Indentation};
use crate::tests::Equivalence;

use super::{Memory, parsers};


#[quickcheck]
fn parse_memory(mut base: Indentation, original: Memory) -> Result<Equivalence<Memory>, String> {
    let mut s: String = Default::default();
    original.fmt(&mut base, &mut s).map_err(|e| e.to_string())?;

    let res = all_consuming(|i| parsers::memory(i, &mut base))(&s)
        .finish()
        .map(|(_, parsed)| Equivalence::of(original, parsed))
        .map_err(|e| e.to_string());
    res
}

