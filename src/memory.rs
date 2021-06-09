//! Memory component

use std::sync::Arc;

use crate::expr;
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


/// The "kind" of a port
#[derive(Copy, Clone, Debug)]
pub enum PortKind {Read, Write, ReadWrite}


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

impl Default for ReadUnderWrite {
    fn default() -> Self {
        Self::Undefined
    }
}

