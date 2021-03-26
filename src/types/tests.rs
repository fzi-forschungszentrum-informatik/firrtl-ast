//! Test related to types

use nom::character::streaming::newline;
use nom::combinator::all_consuming;
use nom::sequence::terminated;

use crate::tests::Equivalence;

use super::{GroundType, parsers};


#[quickcheck]
fn parse_ground_type(original: GroundType) -> Result<Equivalence<GroundType>, String> {
    use nom::Finish;

    let s = format!("{}\n", original);
    let res = all_consuming(terminated(parsers::ground_type, newline))(&s)
        .finish()
        .map(|(_, parsed)| Equivalence::of(original, parsed))
        .map_err(|e| e.to_string());
    res
}

