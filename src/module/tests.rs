// Copyright (c) 2021 FZI Forschungszentrum Informatik
// SPDX-License-Identifier: Apache-2.0
//! Tests related to modules

use nom::Finish;
use nom::combinator::all_consuming;

use quickcheck::{Gen, TestResult, Testable};

use crate::indentation::{DisplayIndented, Indentation};
use crate::named::Named;
use crate::tests::Equivalence;

use super::{Direction, Instance, Module, ParamValue, Port, parsers};


#[quickcheck]
fn parse_module(mut base: Indentation, original: Module) -> Result<TestResult, String> {
    let mut s: String = Default::default();
    original.fmt(&mut base, &mut s).map_err(|e| e.to_string())?;

    let mut mods: Vec<_> = original.referenced_modules().cloned().collect();
    mods.sort_unstable_by_key(|r| r.name().to_string());
    if mods.windows(2).any(|p| p[0].name() == p[1].name()) {
        // We depend on module names to be unique.
        return Ok(TestResult::discard())
    }

    let res = all_consuming(
        |i| parsers::module(
            |n| mods.binary_search_by_key(&n, |r| r.name()).ok().map(|i| mods[i].clone()),
            i,
            &mut base,
        )
    )(&s)
        .finish()
        .map(|(_, parsed)| Equivalence::of(original, parsed).result(&mut Gen::new(0)))
        .map_err(|e| e.to_string());
    res
}


#[quickcheck]
fn parse_param_value(original: ParamValue) -> Result<Equivalence<ParamValue>, String> {
    let s = original.to_string();

    let res = all_consuming(parsers::param_value)(&s)
        .finish()
        .map(|(_, parsed)| Equivalence::of(original, parsed))
        .map_err(|e| e.to_string());
    res
}


#[quickcheck]
fn parse_instance(original: Instance) -> Result<Equivalence<Instance>, String> {
    let s = original.to_string();

    let m = original.module().clone();
    let lookup = move |n: &str| if n == m.name_ref() {
        Some(m.clone())
    } else {
        None
    };

    let res = all_consuming(|i| parsers::instance(&lookup, i))(&s)
        .finish()
        .map(|(_, parsed)| Equivalence::of(original, parsed))
        .map_err(|e| e.to_string());
    res
}


#[quickcheck]
fn parse_port(original: Port) -> Result<Equivalence<Port>, String> {
    let s = original.to_string();

    let res = all_consuming(|i| parsers::port(i))(&s)
        .finish()
        .map(|(_, parsed)| Equivalence::of(original, parsed))
        .map_err(|e| e.to_string());
    res
}


#[quickcheck]
fn parse_direction(original: Direction) -> Result<Equivalence<Direction>, String> {
    let s = original.to_string();
    let res = all_consuming(parsers::direction)(&s)
        .finish()
        .map(|(_, parsed)| Equivalence::of(original, parsed))
        .map_err(|e| e.to_string());
    res
}

