//! Memory component

pub(crate) mod display;
pub(crate) mod parsers;

pub mod common;

#[cfg(test)]
mod tests;

use std::fmt;
use std::sync::Arc;

#[cfg(test)]
use quickcheck::{Arbitrary, Gen};

use crate::expr;
use crate::types;

#[cfg(test)]
use crate::tests::Identifier;

pub use common::{PortDir, ReadUnderWrite};


/// A FIRRTL memory
#[derive(Clone, Debug, PartialEq)]
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
            PortDir::Read       => vec![
                Field::new("data", self.data_type().clone()).flipped(),
                addr_field.clone(),
                en_field.clone(),
                clk_field.clone(),
            ],
            PortDir::Write      => vec![
                Field::new("data", self.data_type().clone()),
                Field::new("mask", mask.clone()),
                addr_field.clone(),
                en_field.clone(),
                clk_field.clone(),
            ],
            PortDir::ReadWrite  => vec![
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
            .map(|p| Field::new(p.name.clone(), port_type(p.dir)).flipped())
            .collect();
        Ok(bundle)
    }
}

#[cfg(test)]
impl Arbitrary for Memory {
    fn arbitrary(g: &mut Gen) -> Self {
        let mut res = Self::new(Identifier::arbitrary(g), types::Type::arbitrary(g), Arbitrary::arbitrary(g));
        res.add_ports((0..u8::arbitrary(g)).map(|_| Arbitrary::arbitrary(g)));
        res.with_read_latency(Arbitrary::arbitrary(g))
            .with_write_latency(Arbitrary::arbitrary(g))
            .with_read_under_write(Arbitrary::arbitrary(g))
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        let d = self.depth();
        let rl = self.read_latency();
        let wl = self.write_latency();
        let ruw = self.read_under_write();

        let res = Identifier::from(self.name.as_ref()).shrink().map({
            let t = self.data_type().clone();
            let p = self.ports.clone();
            move |n| {
                let mut res = Self::new(n, t.clone(), d);
                res.add_ports(p.clone());
                res.with_read_latency(rl)
                    .with_write_latency(wl)
                    .with_read_under_write(ruw)
            }
        }).chain(self.data_type().shrink().map({
            let n = self.name.clone();
            let p = self.ports.clone();
            move |t| {
                let mut res = Self::new(n.clone(), t, d);
                res.add_ports(p.clone());
                res.with_read_latency(rl)
                    .with_write_latency(wl)
                    .with_read_under_write(ruw)
            }
        })).chain(self.depth().shrink().map({
            let n = self.name.clone();
            let t = self.data_type().clone();
            let p = self.ports.clone();
            move |d| {
                let mut res = Self::new(n.clone(), t.clone(), d);
                res.add_ports(p.clone());
                res.with_read_latency(rl)
                    .with_write_latency(wl)
                    .with_read_under_write(ruw)
            }
        })).chain(self.ports.shrink().map({
            let n = self.name.clone();
            let t = self.data_type().clone();
            move |p| {
                let mut res = Self::new(n.clone(), t.clone(), d);
                res.add_ports(p);
                res.with_read_latency(rl)
                    .with_write_latency(wl)
                    .with_read_under_write(ruw)
            }
        })).chain(self.read_latency().shrink().map({
            let mem = self.clone();
            move |l| mem.clone().with_read_latency(l)
        })).chain(self.write_latency().shrink().map({
            let mem = self.clone();
            move |l| mem.clone().with_write_latency(l)
        })).chain(self.read_under_write().shrink().map({
            let mem = self.clone();
            move |ruw| mem.clone().with_read_under_write(ruw)
        }));
        Box::new(res)
    }
}


/// Depth of a memory
type Depth = u64;


/// Read or write latency in clock-cycles
type Latency = u16;


/// Port of a memory
#[derive(Clone, Debug, PartialEq)]
pub struct Port {
    pub name: Arc<str>,
    pub dir: PortDir,
}

impl fmt::Display for Port {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let kind = match self.dir {
            PortDir::Read       => "reader",
            PortDir::Write      => "writer",
            PortDir::ReadWrite  => "readwriter",
        };
        write!(f, "{} => {}", kind, self.name)
    }
}

#[cfg(test)]
impl Arbitrary for Port {
    fn arbitrary(g: &mut Gen) -> Self {
        Self {name: Identifier::arbitrary(g).into(), dir: Arbitrary::arbitrary(g)}
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        let res = Identifier::from(self.name.as_ref()).shrink().map({
            let dir = self.dir;
            move |n| Port {name: n.into(), dir}
        }).chain(self.dir.shrink().map({
            let n = self.name.clone();
            move |dir| Port {name: n.clone(), dir}
        }));
        Box::new(res)
    }
}

