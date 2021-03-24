//! FIRRTL data structure
//!
//! [FIRRTL](https://chisel-lang.org/firrtl/) is a simple HDL
//! register-transfer level.

mod circuit;
mod module;
mod types;

pub use circuit::Circuit;
pub use module::Module;
pub use types::{GroundType, Orientation, OrientedType, Type, Width};

