//! FIRRTL data structure
//!
//! [FIRRTL](https://chisel-lang.org/firrtl/) is a simple register-transfer
//! level HDL.

mod circuit;
mod module;
mod parsers;
mod types;
mod indentation;

#[cfg(test)]
mod tests;


#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

pub use circuit::Circuit;
pub use circuit::parsers::circuit as parse;
pub use module::{Direction, Module, Port};
pub use types::{GroundType, Orientation, OrientedType, Type, TypeEq};

