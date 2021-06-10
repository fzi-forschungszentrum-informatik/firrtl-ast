//! Types and utilities related to FIRRTL statements

use std::sync::Arc;

use crate::expr;
use crate::memory::Memory;
use crate::register::Register;
use crate::module;
use crate::types;


/// Referencable entity
///
/// FIRRTL defines several entities which may be referenced inside an
/// expression.
#[derive(Clone, Debug, PartialEq)]
pub enum Entity {
    Port(Arc<module::Port>),
    Wire{name: Arc<str>, r#type: types::Type},
    Register(Register<Arc<Self>>),
    Node{name: Arc<str>, value: expr::Expression<Arc<Self>>},
    Memory(Memory),
    Instance(module::Instance),
}

impl expr::Reference for Arc<Entity> {
    fn name(&self) -> &str {
        match self.as_ref() {
            Entity::Port(port)      => port.name(),
            Entity::Wire{name, ..}  => name.as_ref(),
            Entity::Register(reg)   => reg.name(),
            Entity::Node{name, ..}  => name.as_ref(),
            Entity::Memory(mem)     => mem.name(),
            Entity::Instance(inst)  => inst.name(),
        }
    }

    fn flow(&self) -> expr::Flow {
        match self.as_ref() {
            Entity::Port(port)      => port.flow(),
            Entity::Wire{..}        => expr::Flow::Duplex,
            Entity::Register(reg)   => reg.flow(),
            Entity::Node{..}        => expr::Flow::Source,
            Entity::Memory(mem)     => mem.flow(),
            Entity::Instance(inst)  => inst.flow(),
        }
    }
}

impl types::Typed for Arc<Entity> {
    type Err = Self;

    type Type = types::Type;

    fn r#type(&self) -> Result<Self::Type, Self::Err> {
        match self.as_ref() {
            Entity::Port(port)          => Ok(port.r#type().clone()),
            Entity::Wire{r#type, ..}    => Ok(r#type.clone()),
            Entity::Register(reg)       => reg.r#type().map_err(|_| self.clone()),
            Entity::Node{value, ..}     => value.r#type().map_err(|_| self.clone()),
            Entity::Memory(mem)         => mem.r#type().map_err(|_| self.clone()),
            Entity::Instance(inst)      => inst.r#type().map_err(|_| self.clone()),
        }
    }
}

