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
use crate::info;
use crate::stmt::Statement;
use crate::types::{self, Type};

pub use parsers::Modules;


/// A hardware block
#[derive(Clone, Debug, PartialEq)]
pub struct Module {
    name: Arc<str>,
    ports: Vec<Arc<Port>>,
    stmts: Option<Vec<Statement>>,
    info: Option<String>,
}

impl Module {
    /// Create a new module
    pub fn new(name: Arc<str>, ports: impl IntoIterator<Item = Arc<Port>>, kind: Kind) -> Self {
        let stmts = match kind {
            Kind::Regular{stmts}    => Some(stmts),
            Kind::External          => None,
        };
        Self {name, ports: ports.into_iter().collect(), stmts, info: Default::default()}
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

    /// Retrieve the module kind
    pub fn kind(&self) -> Kind {
        if let Some(stmts) = self.stmts.clone() {
            Kind::Regular{stmts}
        } else {
            Kind::External
        }
    }

    /// Retrieve the statements in this module
    pub fn statements(&self) -> &[Statement] {
        self.stmts.as_ref().map(|v| v.as_ref()).unwrap_or(&[])
    }

    /// Retrieve all modules referenced from this module via instantiations
    pub fn referenced_modules(&self) -> impl Iterator<Item = &Arc<Self>> {
        self.statements().iter().flat_map(Statement::instantiations).map(Instance::module)
    }

    /// Retrieve a mutable reference of this module's statement list
    ///
    /// For regulsr modules, this function will return a reference to the
    /// module's internal statement list. For external modules, `None` will be
    /// returned.
    pub fn statements_mut(&mut self) -> Option<&mut Vec<Statement>> {
        self.stmts.as_mut()
    }
}

impl info::WithInfo for Module {
    fn info(&self) -> Option<&str> {
        self.info.as_ref().map(AsRef::as_ref)
    }

    fn set_info(&mut self, info: Option<String>) {
        self.info = info
    }
}

impl DisplayIndented for Module {
    fn fmt<W: fmt::Write>(&self, indentation: &mut Indentation, f: &mut W) -> fmt::Result {
        writeln!(
            f,
            "{}{} {}:{}",
            indentation.lock(),
            self.kind().keyword(),
            self.name(),
            info::Info::of(self),
        )?;
        let mut indentation = indentation.sub();
        self.ports().try_for_each(|p| DisplayIndented::fmt(p, &mut indentation, f))?;
        self.statements().iter().try_for_each(|s| DisplayIndented::fmt(s, &mut indentation, f))
    }
}

#[cfg(test)]
impl Arbitrary for Module {
    fn arbitrary(g: &mut Gen) -> Self {
        use crate::tests::Identifier;

        let max_ports = (usize::arbitrary(g) % 16) + 1;
        let name = Identifier::arbitrary(g).into();
        match Kind::arbitrary(g) {
            Kind::Regular{..} => tests::module_with_stmts(
                name,
                std::iter::from_fn(|| Some(Arbitrary::arbitrary(g))),
                max_ports,
            ),
            Kind::External => {
                let mut sub = Gen::new(g.size() / max_ports);
                let ports = (0..max_ports).map(|_| Arbitrary::arbitrary(&mut sub));
                Module::new(name, ports, Kind::External)
            },
        }
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        let p = self.ports.clone();
        let k = self.kind();
        let res = crate::tests::Identifier::from(self.name())
            .shrink()
            .map(move |n| Self::new(n.into(), p.clone(), k.clone()));

        let n = self.name.clone();
        match self.kind() {
            Kind::Regular{..} => {
                // For regular modules, we must maintain that all ports used in
                // statements are defined for the module. Hence, we shrink the
                // statement list and derive the ports from that list.
                let max_stmts = self.statements().len();
                let res = res.chain(self.statements()
                    .to_vec()
                    .shrink()
                    .map(move |s| tests::module_with_stmts(n.clone(), s, usize::MAX))
                    .filter(move |m| m.statements().len() < max_stmts));
                Box::new(res)
            },
            Kind::External => {
                // For external modules, we can just shrink the port list.
                let res = res.chain(self
                    .ports
                    .shrink()
                    .map(move |p| Self::new(n.clone(), p, Kind::External))
                );
                Box::new(res)
            },
        }
    }
}


/// Module kind
///
/// The FIRRTL spec defines multiple kinds of modules.
#[derive(Clone, Debug, PartialEq)]
pub enum Kind {
    /// A regular module
    Regular{stmts: Vec<Statement>},
    /// An external module, usually an interface to some IP or external
    /// VHDL/Verilog.
    External,
}

impl Kind {
    /// Retrieve the keyword associated with the module kind
    pub fn keyword(&self) -> &'static str {
        match self {
            Self::Regular{..}   => "module",
            Self::External      => "extmodule",
        }
    }

    /// Create a new, empty module kind for regular modules
    pub fn empty_regular() -> Self {
        Self::Regular{stmts: Default::default()}
    }

    /// Create a new, empty module kind for external modules
    pub fn empty_external() -> Self {
        Self::External
    }

    /// Retrieve the statements in this module
    pub fn statements(&self) -> &[Statement] {
        match self {
            Self::Regular{stmts}    => stmts.as_ref(),
            Self::External          => &[],
        }
    }
}

impl Default for Kind {
    fn default() -> Self {
        Self::empty_regular()
    }
}

#[cfg(test)]
impl Arbitrary for Kind {
    fn arbitrary(g: &mut Gen) -> Self {
        if g.size() > 0 {
            let opts: [fn() -> Self; 2] = [Self::empty_regular, Self::empty_external];
            g.choose(&opts).unwrap()()
        } else {
            Default::default()
        }
    }
}


/// An I/O port of a module
#[derive(Clone, Debug, PartialEq)]
pub struct Port {
    name: Arc<str>,
    r#type: Type,
    direction: Direction,
    info: Option<String>,
}

impl Port {
    /// Create a new port
    pub fn new(name: impl Into<Arc<str>>, r#type: Type, direction: Direction) -> Self {
        Self {name: name.into(), r#type, direction, info: Default::default()}
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

impl info::WithInfo for Port {
    fn info(&self) -> Option<&str> {
        self.info.as_ref().map(AsRef::as_ref)
    }

    fn set_info(&mut self, info: Option<String>) {
        self.info = info
    }
}

impl fmt::Display for Port {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}: {}{}", self.direction(), self.name(), self.r#type(), info::Info::of(self))
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

