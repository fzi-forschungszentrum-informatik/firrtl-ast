// Copyright (c) 2021 FZI Forschungszentrum Informatik
// SPDX-License-Identifier: Apache-2.0
//! FIRRTL module and associated utilties

pub(crate) mod parsers;

#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

#[cfg(test)]
use quickcheck::{Arbitrary, Gen};

use crate::expr;
use crate::indentation::{DisplayIndented, Indentation};
use crate::info;
use crate::named::Named;
use crate::stmt::Statement;
use crate::types::{self, Type};

pub use parsers::Modules;


/// FIRRTL `module` or `extmodule`
///
/// A `Module` represents a hardware block. Two [Kind]s of `Module`s exist in
/// FIRRTL: `module`s are defined via FIRRTL [Statement]s while `exmodule`s are
/// black boxes and may refer to external definitions such as Verilog sources.
#[derive(Clone, Debug, PartialEq)]
pub struct Module {
    name: Arc<str>,
    ports: Vec<Arc<Port>>,
    kind: Kind,
    info: Option<String>,
}

impl Module {
    /// Create a new module
    pub fn new(name: Arc<str>, ports: impl IntoIterator<Item = Arc<Port>>, kind: Kind) -> Self {
        Self {name, ports: ports.into_iter().collect(), kind, info: Default::default()}
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
    pub fn kind(&self) -> &Kind {
        &self.kind
    }

    /// Retrieve a mutable reference of the module kind
    pub fn kind_mut(&mut self) -> &mut Kind {
        &mut self.kind
    }

    /// Retrieve the statements in this module
    pub fn statements(&self) -> &[Statement] {
        self.kind.statements()
    }

    /// Retrieve all modules referenced from this module via instantiations
    pub fn referenced_modules(&self) -> impl Iterator<Item = &Arc<Self>> {
        self.statements().iter().flat_map(Statement::instantiations).map(Instance::module)
    }
}

impl Named for Module {
    type Name = Arc<str>;

    fn name(&self) -> &Self::Name {
        &self.name
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
        match self.kind() {
            Kind::Regular{stmts} => stmts
                .iter()
                .try_for_each(|s| DisplayIndented::fmt(s, &mut indentation, f)),
            Kind::External{defname, params} => {
                defname.as_ref().map(|n| writeln!(f, "{}defname = {}", indentation.lock(), n)).transpose()?;
                params
                    .iter()
                    .try_for_each(|(k, v)| writeln!(f, "{}parameter {} = {}", indentation.lock(), k, v))
            },
        }
    }
}

#[cfg(test)]
impl Arbitrary for Module {
    fn arbitrary(g: &mut Gen) -> Self {
        use crate::stmt::{self, tests::stmt_exprs};
        use crate::tests::Identifier;

        let name = Identifier::arbitrary(g).into();
        let kind: Kind = Arbitrary::arbitrary(g);
        let ports: Vec<_> = if kind.statements().is_empty() {
            let n = (usize::arbitrary(g) % 16) + 1;
            let mut g = Gen::new(g.size() / n);
            std::iter::from_fn(|| Some(Arbitrary::arbitrary(&mut g))).take(n).collect()
        } else {
            let ports: HashMap<_, _> = kind
                .statements()
                .iter()
                .flat_map(transiter::AutoTransIter::trans_iter)
                .flat_map(stmt_exprs)
                .flat_map(expr::Expression::references)
                .filter_map(|e| if let stmt::Entity::Port(p) = e.as_ref() { Some(p) } else { None })
                .map(|p| (p.name(), p.clone()))
                .collect();
            ports.into_iter().map(|(_, v)| v).collect()
        };

        Module::new(name, ports, kind)
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        let res = crate::tests::Identifier::from(self.name_ref())
            .shrink()
            .map({
                let p = self.ports.clone();
                let k = self.kind().clone();
                move |n| Self::new(n.into(), p.clone(), k.clone())
            })
            .chain(self.kind().shrink().map({
                let n = self.name.clone();
                let p = self.ports.clone();
                move |k| Self::new(n.clone(), p.clone(), k)
            }));

        // We can shrink ports only if we don't have to accomoidate for any
        // statements which could potentially reference them.
        if self.statements().is_empty() {
            let n = self.name.clone();
            let k = self.kind().clone();
            Box::new(res.chain(self.ports.shrink().map(move |p| Self::new(n.clone(), p, k.clone()))))
        } else {
            Box::new(res)
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
    External{defname: Option<Arc<str>>, params: HashMap<Arc<str>, ParamValue>},
}

impl Kind {
    /// Retrieve the keyword associated with the module kind
    pub fn keyword(&self) -> &'static str {
        match self {
            Self::Regular{..}   => "module",
            Self::External{..}  => "extmodule",
        }
    }

    /// Create a new, empty module kind for regular modules
    pub fn empty_regular() -> Self {
        Self::Regular{stmts: Default::default()}
    }

    /// Create a new, empty module kind for external modules
    pub fn empty_external() -> Self {
        Self::External{defname: Default::default(), params: Default::default()}
    }

    /// Retrieve the statements in this module
    pub fn statements(&self) -> &[Statement] {
        match self {
            Self::Regular{stmts}    => stmts.as_ref(),
            Self::External{..}      => &[],
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
        use std::iter::from_fn as fn_iter;

        use crate::stmt::tests::stmts_with_decls;
        use crate::tests::Identifier;

        if g.size() <= 0 {
            return Default::default();
        }

        let opts: [&dyn Fn(&mut Gen) -> Self; 2] = [
            &|g| {
                let n = u8::arbitrary(g) as usize;
                let mut g = Gen::new(g.size() / std::cmp::max(n, 1));
                let stmts = stmts_with_decls(fn_iter(|| Some(Arbitrary::arbitrary(&mut g))).take(n))
                    .collect();
                Self::Regular{stmts}
            },
            &|g| {
                let defname = Option::<Identifier>::arbitrary(g).map(Into::into);
                let n = u8::arbitrary(g) as usize;
                let mut g = Gen::new(g.size() / std::cmp::max(n, 1));
                let params = fn_iter(
                    || Some((Identifier::arbitrary(&mut g).into(), Arbitrary::arbitrary(&mut g)))
                ).take(n).collect();
                Kind::External{defname, params}
            },
        ];
        g.choose(&opts).unwrap()(g)
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        use std::iter::once;

        use crate::stmt::tests::stmts_with_decls;
        use crate::tests::Identifier;

        match self {
            Self::Regular{stmts} => {
                let max_stmts = stmts.len();
                let res = stmts
                    .to_vec()
                    .shrink()
                    .map(|v| stmts_with_decls(v).collect::<Vec<_>>())
                    .filter(move |v| v.len() < max_stmts)
                    .map(|stmts| Self::Regular{stmts});
                Box::new(res)
            },
            Kind::External{defname, params} => {
                let res = defname
                    .as_ref()
                    .map(|n| Identifier::from(n.as_ref()))
                    .shrink()
                    .map({
                        let p = params.clone();
                        move |n| Kind::External{defname: n.map(Into::into), params: p.clone()}
                    });
                if params.len() > 1 {
                    let n = defname.clone();
                    let res = res.chain(params
                        .clone()
                        .into_iter()
                        .map(move |p| Kind::External{defname: n.clone(), params: once(p).collect()})
                    );
                    Box::new(res)
                } else if params.len() > 1 {
                    let res = res.chain(
                        once(Kind::External{defname: defname.clone(), params: Default::default()})
                    );
                    Box::new(res)
                } else {
                    Box::new(res)
                }
            },
        }
    }
}


/// Representation of a parameter value
#[derive(Clone, PartialEq, Debug)]
pub enum ParamValue {Int(i64), Double(f64), String(Arc<str>)}

impl fmt::Display for ParamValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Int(v)    => fmt::Display::fmt(v, f),
            Self::Double(v) => fmt::Display::fmt(v, f),
            Self::String(v) => {
                fmt::Display::fmt(&'"', f)?;
                v.chars().try_for_each(|c| match c {
                    '\n' => write!(f, "\\\n"),
                    '\t' => write!(f, "\\\t"),
                    '"'  => write!(f, "\\\""),
                    '\'' => write!(f, "\\'"),
                    '\\' => write!(f, "\\\\"),
                    c    => fmt::Display::fmt(&c, f),
                })?;
                fmt::Display::fmt(&'"', f)
            },
        }
    }
}

#[cfg(test)]
impl Arbitrary for ParamValue {
    fn arbitrary(g: &mut Gen) -> Self {
        // We decided against considering Double values in our tests. With parse
        // tests, trying to get back the same double is a matter of luck,
        // especially since our formatting will happily format it as an integer
        // if possible.
        let opts: [&dyn Fn(&mut Gen) -> Self; 2] = [
            &|g| Self::Int(Arbitrary::arbitrary(g)),
            &|g| Self::String(crate::tests::ASCII::arbitrary(g).into()),
        ];
        g.choose(&opts).unwrap()(g)
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        use crate::tests::ASCII;

        match self {
            Self::Int(v)    => Box::new(v.shrink().map(Self::Int)),
            Self::Double(v) => Box::new(v.shrink().map(Self::Double)),
            Self::String(v) => Box::new(ASCII::from(v.as_ref()).shrink().map(Into::into).map(Self::String)),
        }
    }
}


/// An I/O port of a [Module]
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

    /// Retrieve the I/O port's type
    pub fn r#type(&self) -> &Type {
        &self.r#type
    }

    /// Retrieve the I/O port's direction
    ///
    /// An I/O port may be either an input or an output. The direction is
    /// generally expressed in terms of the module. Ports with an direction of
    /// [Direction::Input] will be an [expr::Flow::Source] within the context of
    /// the module.
    pub fn direction(&self) -> Direction {
        self.direction
    }
}

impl expr::Reference for Port {
    fn flow(&self) -> Option<expr::Flow> {
        Some(match self.direction {
            Direction::Input  => expr::Flow::Source,
            Direction::Output => expr::Flow::Sink,
        })
    }
}

impl Named for Port {
    type Name = Arc<str>;

    fn name(&self) -> &Self::Name {
        &self.name
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
        let res = crate::tests::Identifier::from(self.name_ref())
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


/// Representation of a [Module] instance
#[derive(Clone, Debug, PartialEq)]
pub struct Instance {
    name: Arc<str>,
    module: Arc<Module>,
}

impl Instance {
    /// Create a new module instance
    pub fn new(name: impl Into<Arc<str>>, module: Arc<Module>) -> Self {
        Self {name: name.into(), module}
    }

    /// Retrieve the instantiated [Module]
    pub fn module(&self) -> &Arc<Module> {
        &self.module
    }
}

impl expr::Reference for Instance {
    fn flow(&self) -> Option<expr::Flow> {
        Some(expr::Flow::Source)
    }
}

impl Named for Instance {
    type Name = Arc<str>;

    fn name(&self) -> &Self::Name {
        &self.name
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

