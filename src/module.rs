//! Module specific definitions and functions

pub mod parsers;

#[cfg(test)]
mod tests;

use std::fmt;
use std::sync::Arc;

#[cfg(test)]
use quickcheck::{Arbitrary, Gen};

use crate::indentation::{DisplayIndented, Indentation};
use crate::types::Type;


/// A hardware block
#[derive(Clone, Debug, PartialEq)]
pub struct Module {
    name: Arc<str>,
    ports: Vec<Arc<Port>>,
}

impl Module {
    /// Create a new module
    pub fn new(name: Arc<str>, ports: impl IntoIterator<Item = Port>) -> Self {
        let mut ports: Vec<_> = ports.into_iter().map(Arc::new).collect();
        ports.sort_unstable_by_key(|p| p.name.clone());

        Self {name, ports}
    }

    /// Retrieve the module's name
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    /// Retrieve the module's I/O ports
    pub fn ports(&self) -> impl Iterator<Item = &Arc<Port>> {
        self.ports.iter()
    }

    /// Retrieve a specific port by its name
    pub fn port_by_name(&self, name: &impl AsRef<str>) -> Option<&Arc<Port>> {
        self.ports.binary_search_by_key(&name.as_ref(), |p| p.name.as_ref()).ok().map(|i| &self.ports[i])
    }
}

impl DisplayIndented for Module {
    fn fmt<W: fmt::Write>(&self, indentation: &mut Indentation, f: &mut W) -> fmt::Result {
        writeln!(f, "{}module {}:", indentation.lock(), self.name())?;
        let mut indentation = indentation.sub();
        self.ports().try_for_each(|p| DisplayIndented::fmt(p, &mut indentation, f))
    }
}

#[cfg(test)]
impl Arbitrary for Module {
    fn arbitrary(g: &mut Gen) -> Self {
        use crate::tests::Identifier;

        let name = Identifier::arbitrary(g).into();

        // We don't just call `arbitrary()` on a `Vec` because we really have to
        // keep the number of ports low. Otherwise, tests will take forever.
        let len = usize::arbitrary(g) % 16;
        let ports = (0..len)
            .map(|_| Port::arbitrary(&mut Gen::new(g.size() / len)));
        Module::new(name, ports)
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        use crate::tests::Identifier;

        let name: Identifier = self.name().into();
        let ports = self.ports.clone();
        let res = std::iter::once(name.clone())
            .chain(name.shrink())
            .flat_map(move |n| ports
                .shrink()
                .map(move |p| Module::new(n.clone().into(), p.into_iter().map(|p| p.as_ref().clone())))
            );
        Box::new(res)
    }
}


/// An I/O port of a module
#[derive(Clone, Debug, PartialEq)]
pub struct Port {
    name: String,
    r#type: Type,
    direction: Direction,
}

impl Port {
    /// Create a new port
    pub fn new(name: String, r#type: Type, direction: Direction) -> Self {
        Self {name, r#type, direction}
    }

    /// Retrieve the I/O port's name
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    /// Retrieve the I/O port's type
    pub fn r#type(&self) -> &Type {
        &self.r#type
    }

    /// Retrieve the I/O port's direction
    ///
    /// An I/O port may be either an input or an output. The direction is
    /// generally expressed in terms of the module. Ports with an direction of
    /// `Input` will be a sink outside the context of the module and a source
    /// within the context of the module, at least at the top level.
    pub fn direction(&self) -> Direction {
        self.direction
    }
}

impl fmt::Display for Port {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}: {}", self.direction(), self.name(), self.r#type())
    }
}

#[cfg(test)]
impl Arbitrary for Port {
    fn arbitrary(g: &mut Gen) -> Self {
        use crate::tests::Identifier;

        Self::new(Identifier::arbitrary(g).to_string(), Arbitrary::arbitrary(g), Arbitrary::arbitrary(g))
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        use crate::tests::Identifier;

        let r#type = self.r#type.clone();
        let direction = self.direction;
        let res = std::iter::once(self.name().into())
            .chain(Identifier::from(self.name()).shrink())
            .flat_map(move |n| r#type.shrink().map(move |t| Self::new(n.to_string(), t, direction)));
        Box::new(res)
    }
}


/// Direction of an I/O port
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Direction {
    Input,
    Output,
}

impl Direction {
    /// Retrieve the keyword associated with the direction value
    pub fn keyword(&self) -> &'static str {
        match self {
            Self::Input => "input",
            Self::Output => "output",
        }
    }
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.keyword(), f)
    }
}

#[cfg(test)]
impl Arbitrary for Direction {
    fn arbitrary(g: &mut Gen) -> Self {
        *g.choose(&[Self::Input, Self::Output]).unwrap()
    }
}

