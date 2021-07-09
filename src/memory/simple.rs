//! "Simple" memories

use std::fmt;
use std::sync::Arc;

use crate::types;

use super::common::ReadUnderWrite;


/// A "simple" FIRRTL memory
///
/// Instances of this type represent either a `cmem` or `smem`.
#[derive(Clone, Debug, PartialEq)]
pub struct Memory {
    name: Arc<str>,
    data_type: types::Type,
    kind: Kind,
}

impl Memory {
    /// Create a new simple memory
    pub fn new(name: impl Into<Arc<str>>, data_type: impl Into<types::Type>, kind: Kind) -> Self {
        Self {name: name.into(), data_type: data_type.into(), kind}
    }

    /// Retrieve the memory's name
    pub fn name(&self) -> &Arc<str> {
        &self.name
    }

    /// Retrieve the kind of simple memory
    pub fn kind(&self) -> Kind {
        self.kind
    }
}

impl types::Typed for Memory {
    type Err = Self;

    type Type = types::Type;

    fn r#type(&self) -> Result<Self::Type, Self::Err> {
        Ok(self.data_type.clone())
    }
}

impl fmt::Display for Memory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let kind = self.kind();
        write!(f, "{} {}: {}", kind.keyword(), self.name(), self.data_type)?;
        if let Kind::Sequential(Some(ruw)) = kind {
            write!(f, " {}", ruw)?;
        }
        Ok(())
    }
}


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

