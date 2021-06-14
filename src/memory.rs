//! Memory component

pub(crate) mod parsers;

use std::fmt;
use std::sync::Arc;

use crate::expr;
use crate::indentation::{DisplayIndented, Indentation};
use crate::types;


/// A FIRRTL memory
#[derive(Clone, Debug)]
pub struct Memory {
    name: Arc<str>,
    data_type: types::Type,
    depth: Depth,
    ports: Vec<Port>,
    read_latency: Latency,
    write_latency: Latency,
    read_under_write: ReadUnderWrite,
}

impl Memory {
    /// Create a new memory
    ///
    /// The memory will be created with the given name, element type and depth
    /// (number of elements). It will have no ports, latencies of zero and
    /// undefined read-under-write behaviour.
    pub fn new(
        name: impl Into<Arc<str>>,
        data_type: impl Into<types::Type>,
        depth: Depth,
    ) -> Self {
        Self {
            name: name.into(),
            data_type: data_type.into(),
            depth,
            ports: Default::default(),
            read_latency: Default::default(),
            write_latency: Default::default(),
            read_under_write: Default::default(),
        }
    }

    /// Retrieve the data type of the memory
    ///
    /// This function returns the type of a single element in the memory.
    pub fn data_type(&self) -> &types::Type {
        &self.data_type
    }

    /// Retrieve the depth, i.e. the number of elements in the memory
    pub fn depth(&self) -> Depth {
        self.depth
    }

    /// Add a port
    ///
    /// This function appends a the given port to the list of ports.
    pub fn add_port(&mut self, port: Port) {
        self.ports.push(port)
    }

    /// Add a number of ports
    ///
    /// This function appends a the given ports, in order, to the list of ports.
    pub fn add_ports(&mut self, ports: impl IntoIterator<Item = Port>) {
        self.ports.extend(ports);
    }

    /// Retrieve the ports
    ///
    /// The returned iterator will yield the ports in the order they were added.
    pub fn ports(&self) -> impl Iterator<Item = &Port> {
        self.ports.iter()
    }

    /// Set the read latency
    pub fn with_read_latency(self, latency: Latency) -> Self {
        Self {read_latency: latency, ..self}
    }

    /// Retrieve the read latency
    pub fn read_latency(&self) -> Latency {
        self.read_latency
    }

    /// Set the write latency
    pub fn with_write_latency(self, latency: Latency) -> Self {
        Self {write_latency: latency, ..self}
    }

    /// Retrieve the write latency
    pub fn write_latency(&self) -> Latency {
        self.write_latency
    }

    /// Set the read-under-write behaviour
    pub fn with_read_under_write(self, behaviour: ReadUnderWrite) -> Self {
        Self {read_under_write: behaviour, ..self}
    }

    /// Retrieve the read-under-write behaviour
    pub fn read_under_write(&self) -> ReadUnderWrite {
        self.read_under_write
    }
}

impl expr::Reference for Memory {
    fn name(&self) -> &str {
        self.name.as_ref()
    }

    fn flow(&self) -> expr::Flow {
        expr::Flow::Source
    }
}

impl types::Typed for Memory {
    type Err = Self;

    type Type = types::Type;

    fn r#type(&self) -> Result<Self::Type, Self::Err> {
        use types::{BundleField as Field, GroundType as GT, Type, required_address_width};

        let addr_field  = Field::new("addr", GT::UInt(Some(required_address_width(self.depth()))));
        let en_field    = Field::new("en", GT::UInt(Some(1)));
        let clk_field   = Field::new("clk", GT::Clock);

        fn mask(t: &Type) -> Type {
            match t {
                Type::GroundType(_) => GT::UInt(Some(1)).into(),
                Type::Vector(v, w)  => Type::Vector(Arc::new(mask(v)), *w),
                Type::Bundle(v)     => v.iter().map(|f| f.clone().with_type(mask(f.r#type()))).collect(),
            }
        }

        let mask = mask(&self.data_type());

        let port_type = |kind| match kind {
            PortKind::Read      => vec![
                Field::new("data", self.data_type().clone()).flipped(),
                addr_field.clone(),
                en_field.clone(),
                clk_field.clone(),
            ],
            PortKind::Write     => vec![
                Field::new("data", self.data_type().clone()),
                Field::new("mask", mask.clone()),
                addr_field.clone(),
                en_field.clone(),
                clk_field.clone(),
            ],
            PortKind::ReadWrite => vec![
                Field::new("wmode", GT::UInt(Some(1))),
                Field::new("rdata", self.data_type().clone()).flipped(),
                Field::new("wdata", self.data_type().clone()),
                Field::new("wmask", mask.clone()),
                addr_field.clone(),
                en_field.clone(),
                clk_field.clone(),
            ],
        };

        let bundle = self
            .ports()
            .map(|p| Field::new(p.name.clone(), port_type(p.kind)).flipped())
            .collect();
        Ok(bundle)
    }
}

impl DisplayIndented for Memory {
    fn fmt<W: fmt::Write>(&self, indentation: &mut Indentation, f: &mut W) -> fmt::Result {
        use expr::Reference;

        writeln!(f, "{}mem {}:", indentation.lock(), self.name())?;
        let mut indentation = indentation.sub();
        writeln!(f, "{}data-type => {}", indentation.lock(), self.data_type())?;
        writeln!(f, "{}depth => {}", indentation.lock(), self.depth())?;
        self.ports().try_for_each(|p| DisplayIndented::fmt(p, &mut indentation, f))?;
        writeln!(f, "{}read-latency => {}", indentation.lock(), self.read_latency())?;
        writeln!(f, "{}write-latency => {}", indentation.lock(), self.write_latency())?;
        writeln!(f, "{}read-under-write => {}", indentation.lock(), self.read_under_write())
    }
}


/// Depth of a memory
type Depth = u64;


/// Read or write latency in clock-cycles
type Latency = u16;


/// Port of a memory
#[derive(Clone, Debug)]
pub struct Port {
    pub name: Arc<str>,
    pub kind: PortKind,
}

impl fmt::Display for Port {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} => {}", self.kind, self.name)
    }
}


/// The "kind" of a port
#[derive(Copy, Clone, Debug)]
pub enum PortKind {Read, Write, ReadWrite}

impl PortKind {
    /// Retrieve the keyword associated with the port kind
    pub fn keyword(&self) -> &'static str {
        match self {
            Self::Read      => "reader",
            Self::Write     => "writer",
            Self::ReadWrite => "readwriter",
        }
    }
}

impl fmt::Display for PortKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.keyword(), f)
    }
}


/// Read-under-write behaviour
#[derive(Copy, Clone, Debug)]
pub enum ReadUnderWrite {
    /// The old value will be read
    Old,
    /// The new value will be read
    New,
    /// The value read is undefined
    Undefined,
}

impl ReadUnderWrite {
    /// Retrieve the keyword associated with the read-under-write behaviour
    pub fn keyword(&self) -> &'static str {
        match self {
            Self::Old       => "old",
            Self::New       => "new",
            Self::Undefined => "undefined",
        }
    }
}

impl Default for ReadUnderWrite {
    fn default() -> Self {
        Self::Undefined
    }
}

impl fmt::Display for ReadUnderWrite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.keyword(), f)
    }
}

