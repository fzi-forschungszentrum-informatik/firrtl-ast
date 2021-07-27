// Copyright (c) 2021 FZI Forschungszentrum Informatik
// SPDX-License-Identifier: Apache-2.0
//! Memory component

pub(crate) mod display;
pub(crate) mod parsers;

pub mod common;
pub mod mem;
pub mod register;
pub mod simple;

#[cfg(test)]
mod tests;

pub use common::{PortDir, ReadUnderWrite};
pub use mem::Memory;
pub use register::Register;

