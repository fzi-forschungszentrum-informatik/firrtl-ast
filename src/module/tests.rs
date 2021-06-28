//! Tests related to modules

use nom::Finish;
use nom::combinator::all_consuming;

use crate::indentation::{DisplayIndented, Indentation};
use crate::tests::Equivalence;

use super::{Direction, Instance, Module, Port, parsers};


#[quickcheck]
fn parse_module(mut base: Indentation, original: Module) -> Result<Equivalence<Module>, String> {
    let mut s: String = Default::default();
    original.fmt(&mut base, &mut s).map_err(|e| e.to_string())?;

    let modules: Vec<_> = original.referenced_modules().cloned().collect();

    let res = all_consuming(
        |i| parsers::module(|n| modules.iter().find(|m| m.name() == n).cloned(), i, &mut base)
    )(&s)
        .finish()
        .map(|(_, parsed)| Equivalence::of(original, parsed))
        .map_err(|e| e.to_string());
    res
}


#[quickcheck]
fn parse_instance(original: Instance) -> Result<Equivalence<Instance>, String> {
    let s = original.to_string();

    let m = original.module().clone();
    let lookup = move |n: &str| if n == m.name() {
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

