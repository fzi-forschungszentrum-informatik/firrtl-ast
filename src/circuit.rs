//! Circuit specific definitions and functions

pub(crate) mod parsers;

#[cfg(test)]
mod tests;

use std::fmt;
use std::sync::Arc;

#[cfg(test)]
use quickcheck::{Arbitrary, Gen};

use crate::indentation;
use crate::module::Module;


/// FIRRTL circuit
///
/// A `Circuit` is the top level construct in FIRRTL. It contains an arbitrary
/// number of modules, one of which is defined as the "top" module.
#[derive(Clone, Debug, PartialEq)]
pub struct Circuit {
    top: Arc<Module>
}

impl Circuit {
    /// Create a new circuit
    pub fn new(top_module: Arc<Module>) -> Self {
        Self {top: top_module}
    }

    /// Get the top level module
    pub fn top_module(&self) -> &Arc<Module> {
        &self.top
    }
}

impl fmt::Display for Circuit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use indentation::DisplayIndented;

        writeln!(f, "circuit {}:", self.top_module().name())?;
        let mut indent = indentation::Indentation::root().sub();
        self.top_module().fmt(&mut indent, f)
    }
}

#[cfg(test)]
impl Arbitrary for Circuit {
    fn arbitrary(g: &mut Gen) -> Self {
        Self::new(Arbitrary::arbitrary(g))
    }
}

