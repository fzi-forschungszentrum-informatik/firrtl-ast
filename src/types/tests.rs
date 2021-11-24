// Copyright (c) 2021 FZI Forschungszentrum Informatik
// SPDX-License-Identifier: Apache-2.0
//! Test related to types

use nom::combinator::all_consuming;

use crate::tests::Equivalence;

use super::{BitWidth, GroundType, Type, combinator, parsers};
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
fn type_partial_eq(lhs: Type, rhs: GroundType) -> Equivalence<bool> {
    Equivalence::of(lhs == rhs, lhs == Type::from(rhs))
}


#[quickcheck]
fn dummy_combine_self(t: Type) -> Result<Equivalence<Type>, (Type, Type)> {
    DummyCombinator()
        .combine(&t, &t)
        .map_err(|(l, r)| (l.clone(), r.clone()))
        .map(|c| Equivalence::of(t, c))
}


#[quickcheck]
fn bitwidth_max_combine_self(width: BitWidth) -> Result<Equivalence<BitWidth>, (BitWidth, BitWidth)> {
    combinator::FnWidth::from(|l, r| Some(std::cmp::max(l, r)))
        .combine(&width, &width)
        .map_err(|(l, r)| (l.clone(), r.clone()))
        .map(|c| Equivalence::of(width, c))
}


#[quickcheck]
fn bitwidth_min_combine_self(width: BitWidth) -> Result<Equivalence<BitWidth>, (BitWidth, BitWidth)> {
    combinator::FnWidth::from(|l, r| Some(std::cmp::min(l, r)))
        .combine(&width, &width)
        .map_err(|(l, r)| (l.clone(), r.clone()))
        .map(|c| Equivalence::of(width, c))
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
