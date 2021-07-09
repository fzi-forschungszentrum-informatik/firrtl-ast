//! "Simple" memories

use super::common::ReadUnderWrite;


/// Kind of simple memory
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Kind {
    /// Combinatory memory, i.e. a `cmem`
    Combinatory,
    /// Sequential memory, i.e. an `smem`
    Sequential(Option<ReadUnderWrite>),
}

impl Kind {
    /// Retrieve the keyword associated with the memory kind
    pub fn keyword(&self) -> &'static str {
        match self {
            Self::Combinatory   => "cmem",
            Self::Sequential(_) => "smem",
        }
    }
}

