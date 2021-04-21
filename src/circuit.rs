//! Circuit specific definitions and functions

pub mod parsers;

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
#[derive(Clone, Debug)]
pub struct Circuit {
    modules: Vec<Arc<Module>>,
    top: Arc<Module>
}

impl Circuit {
    /// Create a new circuit
    pub fn new(top_module: Arc<Module>, modules: impl IntoIterator<Item = Arc<Module>>) -> Self {
        Self {top: top_module, modules: modules.into_iter().collect()}
    }

    /// Get an iterator over all modules
    pub fn modules(&self) -> impl Iterator<Item = &Arc<Module>> {
        self.modules.iter()
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
        self.modules().try_for_each(|m| m.fmt(&mut indent, f))
    }
}

#[cfg(test)]
impl Arbitrary for Circuit {
    fn arbitrary(g: &mut Gen) -> Self {
        let top: Arc<Module> = Arbitrary::arbitrary(g);

        // We don't just call `arbitrary()` on a `Vec` because we really have to
        // keep the number of ports low. Otherwise, tests will take forever.
        let len = usize::arbitrary(g) % 16;
        let mods = std::iter::once(top.clone())
            .chain((0..len).map(|_| Arbitrary::arbitrary(&mut Gen::new(g.size() / len))));
        Self::new(top, mods)
    }
}

