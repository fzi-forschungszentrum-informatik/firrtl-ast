//! Test related to types

use nom::combinator::all_consuming;

use crate::tests::Equivalence;

use super::{GroundType, Type, TypeExt, combinator, parsers};
use combinator::Combinator;


#[quickcheck]
fn parse_ground_type(original: GroundType) -> Result<Equivalence<GroundType>, String> {
    use nom::Finish;

    let s = original.to_string();
    let res = all_consuming(parsers::ground_type)(&s)
        .finish()
        .map(|(_, parsed)| Equivalence::of(original, parsed))
        .map_err(|e| e.to_string());
    res
}


#[quickcheck]
fn parse_type(original: Type) -> Result<Equivalence<Type>, String> {
    use nom::Finish;

    let s = original.to_string();
    let res = all_consuming(parsers::r#type)(&s)
        .finish()
        .map(|(_, parsed)| Equivalence::of(original, parsed))
        .map_err(|e| e.to_string());
    res
}


#[quickcheck]
fn passive_oriented_eq(base: Type) -> Equivalence<bool> {
    Equivalence::of(base.is_passive(), super::OrientedType::from(&base).is_passive())
}


#[quickcheck]
fn dummy_combine_self(t: Type) -> Result<Equivalence<Type>, (Type, Type)> {
    DummyCombinator()
        .combine(&t, &t)
        .map_err(|(l, r)| (l.clone(), r.clone()))
        .map(|c| Equivalence::of(t, c))
}


struct DummyCombinator();

impl Combinator<GroundType> for DummyCombinator {
    fn combine<'a>(
        &self,
        lhs: &'a GroundType,
        _: &'a GroundType
    ) -> Result<GroundType, (&'a GroundType, &'a GroundType)> {
        Ok(lhs.clone())
    }
}
