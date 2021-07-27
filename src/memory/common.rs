// Copyright (c) 2021 FZI Forschungszentrum Informatik
// SPDX-License-Identifier: Apache-2.0
//! Common types and utilities

use std::fmt;

#[cfg(test)]
use quickcheck::{Arbitrary, Gen};


/// Read-under-write behaviour
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ReadUnderWrite {
    /// The old value will be read
    Old,
    /// The new value will be read
    New,
    /// The value read is undefined
    Undefined,
}

impl ReadUnderWrite {
    /// Retrieve the keyword associated with the read-under-write behaviour
    pub fn keyword(&self) -> &'static str {
        match self {
            Self::Old       => "old",
            Self::New       => "new",
            Self::Undefined => "undefined",
        }
    }
}

impl Default for ReadUnderWrite {
    fn default() -> Self {
        Self::Undefined
    }
}

impl fmt::Display for ReadUnderWrite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.keyword(), f)
    }
}

#[cfg(test)]
impl Arbitrary for ReadUnderWrite {
    fn arbitrary(g: &mut Gen) -> Self {
        g.choose(&[Self::Old, Self::New, Self::Undefined]).unwrap().clone()
    }
}


/// The "kind" of a port
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PortDir {Read, Write, ReadWrite}

#[cfg(test)]
impl Arbitrary for PortDir {
    fn arbitrary(g: &mut Gen) -> Self {
        g.choose(&[Self::Read, Self::Write, Self::ReadWrite]).unwrap().clone()
    }
}

