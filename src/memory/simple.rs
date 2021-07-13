//! "Simple" memories

use std::fmt;
use std::sync::Arc;

use crate::expr;
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

