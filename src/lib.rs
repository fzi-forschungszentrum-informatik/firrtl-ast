//! FIRRTL data structure
//!
//! [FIRRTL](https://chisel-lang.org/firrtl/) is a simple HDL
//! register-transfer level.

mod circuit;
mod module;
mod parsers;
mod types;

pub use circuit::Circuit;
pub use module::{Direction, Module, Port};
pub use types::{GroundType, Orientation, OrientedType, Type, TypeEq, Width};

