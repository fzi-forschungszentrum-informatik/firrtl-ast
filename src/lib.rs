// Copyright (c) 2021 FZI Forschungszentrum Informatik
// SPDX-License-Identifier: Apache-2.0
//! FIRRTL data structure
//!
//! [FIRRTL](https://chisel-lang.org/firrtl/) is a simple register-transfer
//! level HDL.

mod display;
mod indentation;
mod parsers;

pub mod circuit;
pub mod error;
pub mod expr;
pub mod info;
pub mod memory;
pub mod module;
pub mod named;
pub mod stmt;
pub mod types;

#[cfg(test)]
mod tests;


#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

pub use circuit::Circuit;
pub use expr::Expression;
pub use memory::{Memory, Register};
pub use module::Module;
pub use named::Named;
pub use types::{GroundType, Type};

