//! Utilities for implementation of Display

use std::fmt;

use super::BitWidth;


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

