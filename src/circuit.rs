// Copyright (c) 2021 FZI Forschungszentrum Informatik
// SPDX-License-Identifier: Apache-2.0
//! Circuit specific definitions and functions

pub(crate) mod parsers;

#[cfg(test)]
mod tests;

use std::fmt;
use std::sync::Arc;

#[cfg(test)]
use quickcheck::{Arbitrary, Gen};

use crate::error::ParseError;
use crate::indentation;
use crate::info::{self, WithInfo};
use crate::module::Module;

pub use parsers::{circuit as parse, consumer};


/// FIRRTL circuit
///
/// A `Circuit` is the top level construct in FIRRTL. It contains an arbitrary
/// number of modules, one of which is defined as the "top" module.
#[derive(Clone, Debug, PartialEq)]
pub struct Circuit {
    top: Arc<Module>,
    info: Option<String>,
}

impl Circuit {
    /// Create a new circuit
    pub fn new(top_module: Arc<Module>) -> Self {
        Self {top: top_module, info: Default::default()}
    }

    /// Get the top level module
    pub fn top_module(&self) -> &Arc<Module> {
        &self.top
    }

    /// Parse a circuit from an object implementing Read
    ///
    /// This function parses a circuit from the given `Read`, e.g. a `File`.
    ///
    /// # Note
    ///
    /// This function reads the entire source into a separate buffer in memory.
    /// Consider using `parse` if the source is in memory already.
    pub fn from_read(mut read: impl std::io::Read) -> Result<Self, ParseError> {
        let mut buf = Default::default();
        read.read_to_string(&mut buf)?;
        parse(buf.as_ref())
    }
}

impl WithInfo for Circuit {
    fn info(&self) -> Option<&str> {
        self.info.as_ref().map(AsRef::as_ref)
    }

    fn set_info(&mut self, info: Option<String>) {
        self.info = info
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

        writeln!(f, "circuit {}:{}", self.top_module().name(), info::Info::of(self))?;
        let mut indent = indentation::Indentation::root().sub();
        fmt_module(&mut done, &mut indent, self.top_module(), f)
    }
}

#[cfg(test)]
impl Arbitrary for Circuit {
    fn arbitrary(g: &mut Gen) -> Self {
        Self::new(Arbitrary::arbitrary(g))
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(self.top.shrink().map(Self::new))
    }
}


/// Iterator adapter/wrapper for creating a circuit
///
/// Instances of this type wrap an iterator over `Module`s. It allows iterating
/// over the modules yielded by the inner iterator transparently, seeking out
/// the top module for a target circuit by name.
#[derive(Clone, Debug)]
pub struct ModuleConsumer<I: Iterator<Item = Result<Arc<Module>, E>>, E> {
    top_module: TopState,
    info: Option<String>,
    modules: I,
}

impl<I, E> ModuleConsumer<I, E>
where I: Iterator<Item = Result<Arc<Module>, E>>,
      E: Into<ParseError>,
{
    /// Create a new adapter for the given target top module name
    ///
    /// The adapter will allow constructing a `Circuit` with a top-module with
    /// the given `top_name`, provided that `modules` will yield such a module.
    /// The constructed `Circuit` with the given `info`. Note that `None` is a
    /// valid choice, e.g. if the `info` is to be set later.
    pub fn new(top_name: impl Into<String>, info: impl Into<Option<String>>, modules: I) -> Self {
        Self {top_module: TopState::Name(top_name.into()), info: info.into(), modules}
    }

    /// Retrieve the circuit
    ///
    /// If the top module was collected, this function returns the circuit,
    /// otherwise `None` will be returned.
    pub fn circuit(&self) -> Option<Circuit> {
        if let TopState::Module(m) = &self.top_module {
            Some(Circuit::new(m.clone()).with_info(self.info.clone()))
        } else {
            None
        }
    }

    /// Try to create the requested circuit, consuming the iterator
    pub fn into_circuit(mut self) -> Result<Circuit, ParseError> {
        let info = self.info;
        match self.top_module {
            TopState::Name(n)   => self
                .modules
                .find(|m| m.as_ref().ok().map(|m| m.name() == n).unwrap_or(true))
                .map(|r| r.map_err(Into::into))
                .unwrap_or_else(|| Err("top module not found".to_owned().into())),
            TopState::Module(m) => Ok(m),
        }.map(|m| Circuit::new(m).with_info(info))
    }
}

impl<I: Iterator<Item = Result<Arc<Module>, E>>, E> Iterator for ModuleConsumer<I, E> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let res = self.modules.next();
        if let (Some(Ok(m)), TopState::Name(n)) = (res.as_ref(), &self.top_module) {
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

