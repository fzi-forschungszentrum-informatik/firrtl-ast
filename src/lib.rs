//! FIRRTL data structure
//!
//! [FIRRTL](https://chisel-lang.org/firrtl/) is a simple register-transfer
//! level HDL.

mod indentation;
mod parsers;

pub mod circuit;
pub mod expr;
pub mod memory;
pub mod module;
pub mod types;

#[cfg(test)]
mod tests;


#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

pub use circuit::Circuit;
pub use circuit::parsers::circuit as parse;
pub use expr::Expression;
pub use memory::Memory;
pub use module::{Direction, Module, ModuleInstance, Port};
pub use types::{GroundType, Orientation, OrientedType, Type, TypeExt};

