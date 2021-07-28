// Copyright (c) 2021 FZI Forschungszentrum Informatik
// SPDX-License-Identifier: Apache-2.0
//! "Simple" memories

use std::fmt;
use std::sync::Arc;

#[cfg(test)]
use quickcheck::{Arbitrary, Gen};

use crate::expr;
use crate::named::Named;
use crate::types;

use super::common::{PortDir, ReadUnderWrite};


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

    /// Retrieve the kind of simple memory
    pub fn kind(&self) -> Kind {
        self.kind
    }
}

impl Named for Memory {
    type Name = Arc<str>;

    fn name(&self) -> &Self::Name {
        &self.name
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
            write!(f, ", {}", ruw)?;
        }
        Ok(())
    }
}

#[cfg(test)]
impl Arbitrary for Memory {
    fn arbitrary(g: &mut Gen) -> Self {
        use crate::tests::Identifier;

        // The type of a memory must always be a vector type
        Self::new(
            Identifier::arbitrary(g),
            types::Type::Vector(Arbitrary::arbitrary(g), Arbitrary::arbitrary(g)),
            Arbitrary::arbitrary(g)
        )
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        use crate::tests::Identifier;

        let k = self.kind();
        let res = Identifier::from(self.name_ref())
            .shrink()
            .map({
                let t = self.data_type.clone();
                move |n| Self::new(n, t.clone(), k)
            })
            .chain(self.data_type.shrink().map({
                let n = self.name.clone();
                move |t| Self::new(n.clone(), t, k)
            }))
            .chain(k.shrink().map({
                let n = self.name.clone();
                let t = self.data_type.clone();
                move |k| Self::new(n.clone(), t.clone(), k)
            }));
        Box::new(res)
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

#[cfg(test)]
impl Arbitrary for Kind {
    fn arbitrary(g: &mut Gen) -> Self {
        let opts: [&dyn Fn(&mut Gen) -> Self; 2] = [
            &|_| Self::Combinatory,
            &|g| Self::Sequential(Arbitrary::arbitrary(g)),
        ];
        g.choose(&opts).unwrap()(g)
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        match self {
            Self::Combinatory   => Box::new(std::iter::empty()),
            Self::Sequential(b) => Box::new(b.shrink().map(Self::Sequential)),
        }
    }
}


/// A port for a simple memory
#[derive(Clone, Debug, PartialEq)]
pub struct Port<R: expr::Reference> {
    name: Arc<str>,
    mem: Arc<Memory>,
    dir: Option<PortDir>,
    addr: expr::Expression<R>,
    clock: expr::Expression<R>,
}

impl<R: expr::Reference> Port<R> {
    /// Create a new memory port
    pub fn new(
        name: impl Into<Arc<str>>,
        mem: Arc<Memory>,
        dir: Option<PortDir>,
        addr: expr::Expression<R>,
        clock: expr::Expression<R>,
    ) -> Self {
        Self {name: name.into(), mem, dir, addr, clock}
    }

    /// Retrieve the memory associated with this port
    pub fn memory(&self) -> &Arc<Memory> {
        &self.mem
    }

    /// Retrieve the direction of this port
    pub fn direction(&self) -> Option<PortDir> {
        self.dir
    }

    /// Retrieve the address
    pub fn address(&self) -> &expr::Expression<R> {
        &self.addr
    }

    /// Retrieve the clock driving this port
    pub fn clock(&self) -> &expr::Expression<R> {
        &self.clock
    }
}

impl<R: expr::Reference> types::Typed for Port<R> {
    type Err = Arc<Memory>;

    type Type = types::Type;

    fn r#type(&self) -> Result<Self::Type, Self::Err> {
        self.memory()
            .r#type()
            .map_err(Arc::new)
            .and_then(|t| t.vector_base().map(|t| t.as_ref().clone()).ok_or(self.mem.clone()))
    }
}

impl<R: expr::Reference> expr::Reference for Port<R> {
    fn name(&self) -> &str {
        self.name.as_ref()
    }

    fn flow(&self) -> Option<expr::Flow> {
        self.dir.map(|d| match d {
            PortDir::Read       => expr::Flow::Source,
            PortDir::Write      => expr::Flow::Sink,
            PortDir::ReadWrite  => expr::Flow::Duplex,
        })
    }
}

impl<R: expr::Reference> fmt::Display for Port<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use expr::Reference;

        let mdir = match self.direction() {
            Some(PortDir::Read)         => "read",
            Some(PortDir::Write)        => "write",
            Some(PortDir::ReadWrite)    => "rdwr",
            None                        => "infer",
        };
        write!(
            f,
            "{} mport {} = {}[{}], {}",
            mdir,
            self.name(),
            self.memory().name(),
            self.address(),
            self.clock()
        )
    }
}

#[cfg(test)]
impl<R: expr::tests::TypedRef + Clone + 'static> Arbitrary for Port<R> {
    fn arbitrary(g: &mut Gen) -> Self {
        use crate::tests::Identifier;
        use expr::tests::{expr_with_type, source_flow};

        Self::new(
            Identifier::arbitrary(g),
            Arbitrary::arbitrary(g),
            Arbitrary::arbitrary(g),
            expr_with_type(types::GroundType::UInt(Arbitrary::arbitrary(g)), source_flow(g), g),
            expr_with_type(types::GroundType::Clock, source_flow(g), g),
        )
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        use crate::tests::Identifier;
        use expr::Reference;

        let res = Identifier::from(self.name())
            .shrink()
            .map({
                let m = self.memory().clone();
                let d = self.direction().clone();
                let a = self.address().clone();
                let c = self.clock().clone();
                move |n| Self::new(n, m.clone(), d, a.clone(), c.clone())
            })
            .chain(self.memory().shrink().map({
                let n = self.name.clone();
                let d = self.direction().clone();
                let a = self.address().clone();
                let c = self.clock().clone();
                move |m| Self::new(n.clone(), m, d, a.clone(), c.clone())
            }))
            .chain(self.direction().shrink().map({
                let n = self.name.clone();
                let m = self.memory().clone();
                let a = self.address().clone();
                let c = self.clock().clone();
                move |d| Self::new(n.clone(), m.clone(), d, a.clone(), c.clone())
            }));
        Box::new(res)
    }
}

