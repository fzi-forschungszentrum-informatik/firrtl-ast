//! FIRRTL data structure
//!
//! [FIRRTL](https://chisel-lang.org/firrtl/) is a simple HDL
//! register-transfer level.

mod circuit;
mod module;

pub use circuit::Circuit;
pub use module::Module;

