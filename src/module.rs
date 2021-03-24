//! Module specific definitions and functions

use std::sync::Arc;

use crate::types::Type;


/// A hardware block
#[derive(Clone, Debug)]
pub struct Module {
    name: Arc<str>,
    ports: Vec<Arc<Port>>,
}

impl Module {
    /// Create a new module
    pub fn new(name: String, ports: impl IntoIterator<Item = (String, Type, Direction)>) -> Self {
        let name: Arc<str> = name.into();

        let mut ports: Vec<_> = ports
            .into_iter()
            .map(|(n, t, d)| Arc::new(Port {module: name.clone(), name: n, r#type: t, direction: d}))
            .collect();
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


/// An I/O port of a module
#[derive(Clone, Debug)]
pub struct Port {
    module: Arc<str>,
    name: String,
    r#type: Type,
    direction: Direction,
}

impl Port {
    /// Retrieve the module this I/O port is associated with
    pub fn module(&self) -> &str {
        self.module.as_ref()
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


/// Direction of an I/O port
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Direction {
    Input,
    Output,
}

