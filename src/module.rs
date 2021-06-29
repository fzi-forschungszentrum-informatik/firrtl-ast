//! Module specific definitions and functions

pub(crate) mod parsers;

#[cfg(test)]
mod tests;

use std::fmt;
use std::sync::Arc;

#[cfg(test)]
use quickcheck::{Arbitrary, Gen};

use crate::expr;
use crate::indentation::{DisplayIndented, Indentation};
use crate::types::{self, Type};


/// A hardware block
#[derive(Clone, Debug, PartialEq)]
pub struct Module {
    name: Arc<str>,
    ports: Vec<Arc<Port>>,
}

impl Module {
    /// Create a new module
    pub fn new(name: Arc<str>, ports: impl IntoIterator<Item = Arc<Port>>) -> Self {
        Self {name, ports: ports.into_iter().collect()}
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
        self.ports().find(|p| p.name.as_ref() == name.as_ref())
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
            .map(|_| Arbitrary::arbitrary(&mut Gen::new(g.size() / len)));
        Module::new(name, ports)
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        let p = self.ports.clone();
        let res = crate::tests::Identifier::from(self.name())
            .shrink()
            .map(move |n| Self::new(n.into(), p.clone()))
            .chain({
                let n = self.name.clone();
                self.ports
                    .shrink()
                    .map(move |p| Self::new(n.clone(), p))
            });
        Box::new(res)
    }
}


/// An I/O port of a module
#[derive(Clone, Debug, PartialEq)]
pub struct Port {
    name: Arc<str>,
    r#type: Type,
    direction: Direction,
}

impl Port {
    /// Create a new port
    pub fn new(name: impl Into<Arc<str>>, r#type: Type, direction: Direction) -> Self {
        Self {name: name.into(), r#type, direction}
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

impl expr::Reference for Port {
    fn name(&self) -> &str {
        self.name.as_ref()
    }

    fn flow(&self) -> expr::Flow {
        match self.direction {
            Direction::Input  => expr::Flow::Source,
            Direction::Output => expr::Flow::Sink,
        }
    }
}

impl types::Typed for Port {
    type Err = Self;

    type Type = Type;

    fn r#type(&self) -> Result<Self::Type, Self::Err> {
        Ok(self.r#type().clone())
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
        let d = self.direction;
        let res = crate::tests::Identifier::from(self.name())
            .shrink()
            .map({
                let t = self.r#type.clone();
                move |n| Self::new(n.to_string(), t.clone(), d)
            })
            .chain({
                let n = self.name.clone();
                self.r#type().shrink().map(move |t| Self::new(n.clone(), t, d))
            })
            .chain({
                let n = self.name.clone();
                let t = self.r#type().clone();
                self.direction.shrink().map(move |d| Self::new(n.clone(), t.clone(), d))
            });
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


/// Representation of a module instance
///
#[derive(Clone, Debug, PartialEq)]
pub struct Instance {
    name: Arc<str>,
    module: Arc<Module>,
}

impl Instance {
    /// Create a new module instance
    ///
    pub fn new(name: impl Into<Arc<str>>, module: Arc<Module>) -> Self {
        Self {name: name.into(), module}
    }

    /// Retrieve the instantiated module
    ///
    pub fn module(&self) -> &Arc<Module> {
        &self.module
    }
}

impl expr::Reference for Instance {
    fn name(&self) -> &str {
        self.name.as_ref()
    }

    fn flow(&self) -> expr::Flow {
        expr::Flow::Source
    }
}

impl types::Typed for Instance {
    type Err = Self;

    type Type = Type;

    fn r#type(&self) -> Result<Self::Type, Self::Err> {
        use types::{BundleField, Orientation};

        fn orientation(dir: Direction) -> Orientation {
            match dir {
                Direction::Input  => Orientation::Flipped,
                Direction::Output => Orientation::Normal,
            }
        }

        let res = self.module.ports().map(|p| BundleField::new(p.name.clone(), p.r#type().clone())
            .with_orientation(orientation(p.direction()))
        ).collect();

        Ok(res)
    }
}

impl fmt::Display for Instance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use expr::Reference;

        write!(f, "inst {} of {}", self.name(), self.module().name())
    }
}

#[cfg(test)]
impl Arbitrary for Instance {
    fn arbitrary(g: &mut Gen) -> Self {
        use crate::tests::Identifier;

        Self::new(Identifier::arbitrary(g), Arbitrary::arbitrary(g))
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        let n = self.name.clone();
        let m = self.module().clone();

        let res = crate::tests::Identifier::from(n.as_ref())
            .shrink()
            .map(move |n| Self::new(n, m.clone()))
            .chain(self.module().shrink().map(move |m| Self::new(n.clone(), m)));
        Box::new(res)
    }
}

