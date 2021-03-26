//! FIRRTL data structure
//!
//! [FIRRTL](https://chisel-lang.org/firrtl/) is a simple HDL
//! register-transfer level.

mod circuit;
mod module;
mod types;

#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

pub use circuit::Circuit;
pub use module::{Direction, Module, Port};
pub use types::{GroundType, Orientation, OrientedType, Type, TypeEq, Width};

