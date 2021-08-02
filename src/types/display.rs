// Copyright (c) 2021 FZI Forschungszentrum Informatik
// SPDX-License-Identifier: Apache-2.0
//! Utilities for implementation of Display

use std::fmt;

use super::{BitWidth, SBits};


/// Utility type for formatting bit widths
pub struct Width {
    width: BitWidth,
}

impl From<&BitWidth> for Width {
    fn from(width: &BitWidth) -> Self {
        Self::from(*width)
    }
}

impl From<BitWidth> for Width {
    fn from(width: BitWidth) -> Self {
        Self {width}
    }
}

impl fmt::Display for Width {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.width.map(|w| write!(f, "<{}>", w)).unwrap_or(Ok(()))
    }
}


/// Utility type for formatting point offsets
pub struct PointOff {
    off: Option<SBits>,
}

impl From<&Option<SBits>> for PointOff {
    fn from(off: &Option<SBits>) -> Self {
        Self::from(*off)
    }
}

impl From<Option<SBits>> for PointOff {
    fn from(off: Option<SBits>) -> Self {
        Self {off}
    }
}

impl fmt::Display for PointOff {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.off.map(|w| write!(f, "<<{}>>", w)).unwrap_or(Ok(()))
    }
}

