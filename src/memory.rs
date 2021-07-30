// Copyright (c) 2021 FZI Forschungszentrum Informatik
// SPDX-License-Identifier: Apache-2.0
//! Memory components
//!
//! FIRRTL knows several different constructs for compoenents which may hold
//! data, i.e. forms of memory:
//!  * [Register]s (`reg`s) are registers which hold one value, are clocked and
//!    can be reset.
//!  * [mem::Memory]s (`mem`s)are blocks of addressable memory with possibly
//!    multiple ports driven by a common clock. In an FPGA, these usually are
//!    usually "external", i.e. not synthesized, components.
//!  * [simple::Memory]s (`smem`s, `cmem`s) are also addressable blocks of
//!    memory. However, ports are conjured separately and may have different
//!    clocks. These are usually used for buffers inside a component.

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

