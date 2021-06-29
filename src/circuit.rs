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
        use std::collections::HashSet;
        use indentation::{DisplayIndented, Indentation};

        // Format a module and all its dependencies, if it wasn't yet formatted
        fn fmt_module<'a>(
            done: &mut HashSet<&'a str>,
            indent: &mut Indentation,
            module: &'a Module,
            f: &mut fmt::Formatter<'_>,
        ) -> fmt::Result {
            if done.insert(module.name()) {
                module.referenced_modules().try_for_each(|m| fmt_module(done, indent, m, f))?;
                module.fmt(indent, f)
            } else {
                Ok(())
            }
        }

        let mut done = Default::default();

        writeln!(f, "circuit {}:", self.top_module().name())?;
        let mut indent = indentation::Indentation::root().sub();
        fmt_module(&mut done, &mut indent, self.top_module(), f)
    }
}

#[cfg(test)]
impl Arbitrary for Circuit {
    fn arbitrary(g: &mut Gen) -> Self {
        Self::new(Arbitrary::arbitrary(g))
    }
}


/// Iterator adapter/wrapper for creating a circuit
///
/// Instances of this type wrap an iterator over `Module`s. It allows iterating
/// over the modules yielded by the inner iterator transparently, seeking out
/// the top module for a target circuit by name.
#[derive(Clone, Debug)]
pub struct ModuleConsumer<I: Iterator<Item = Arc<Module>>> {
    top_module: TopState,
    modules: I,
}

impl<I: Iterator<Item = Arc<Module>>> ModuleConsumer<I> {
    /// Create a new adapter for the given target top module name
    pub fn new(top_name: impl Into<String>, modules: I) -> Self {
        Self {top_module: TopState::Name(top_name.into()), modules}
    }

    /// Retrieve the circuit
    ///
    /// If the top module was collected, this function returns the circuit,
    /// otherwise `None` will be returned.
    pub fn circuit(&self) -> Option<Circuit> {
        if let TopState::Module(m) = &self.top_module {
            Some(Circuit::new(m.clone()))
        } else {
            None
        }
    }

    /// Try to create the requested circuit, consuming the iterator
    pub fn into_circuit(mut self) -> Option<Circuit> {
        match self.top_module {
            TopState::Name(n)   => self.modules.find(|m| m.name() == n),
            TopState::Module(m) => Some(m),
        }.map(Circuit::new)
    }
}

impl<I: Iterator<Item = Arc<Module>>> Iterator for ModuleConsumer<I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let res = self.modules.next();
        if let (Some(m), TopState::Name(n)) = (res.as_ref(), &self.top_module) {
            if n == m.name() {
                self.top_module = TopState::Module(m.clone())
            }
        }
        res
    }
}


#[derive(Clone, Debug)]
enum TopState {
    Name(String),
    Module(Arc<Module>),
}

