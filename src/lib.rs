//! FIRRTL data structure
//!
//! [FIRRTL](https://chisel-lang.org/firrtl/) is a simple register-transfer
//! level HDL.

mod display;
mod indentation;
mod parsers;

pub mod circuit;
pub mod expr;
pub mod info;
pub mod memory;
pub mod module;
pub mod register;
pub mod stmt;
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
pub use module::Module;
pub use register::Register;
pub use types::{GroundType, Type};

