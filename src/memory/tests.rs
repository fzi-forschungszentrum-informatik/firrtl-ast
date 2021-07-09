//! Tests related to memories

use nom::Finish;
use nom::combinator::all_consuming;

use crate::indentation::{DisplayIndented, Indentation};
use crate::tests::{Equivalence, Identifier};

use super::{Memory, Register, display::MemoryDecl, parsers, simple};


#[quickcheck]
fn parse_memory(
    mut base: Indentation,
    original: Memory
) -> Result<Equivalence<(Memory, Option<String>)>, String> {
    let mut s: String = Default::default();
    MemoryDecl(&original, Default::default()).fmt(&mut base, &mut s).map_err(|e| e.to_string())?;

    let res = all_consuming(|i| parsers::memory(i, &mut base))(&s)
        .finish()
        .map(|(_, parsed)| Equivalence::of((original, None), parsed))
        .map_err(|e| e.to_string());
    res
}


#[quickcheck]
fn parse_simple_mem(original: simple::Memory) -> Result<Equivalence<simple::Memory>, String> {
    let s = original.to_string();

    let res = all_consuming(parsers::simple_mem)(&s)
        .finish()
        .map(|(_, parsed)| Equivalence::of(original, parsed))
        .map_err(|e| e.to_string());
    res
}


#[quickcheck]
fn parse_register(original: Register<Identifier>) -> Result<Equivalence<Register<Identifier>>, String> {
    let s = original.to_string();
    let res = all_consuming(|i| parsers::register(|s| Some(s.into()), i))(&s)
        .finish()
        .map(|(_, parsed)| Equivalence::of(original, parsed))
        .map_err(|e| e.to_string());
    res
}

