// Copyright (c) 2021 FZI Forschungszentrum Informatik
// SPDX-License-Identifier: Apache-2.0
//! Referenable entities

use std::sync::Arc;

#[cfg(test)]
use quickcheck::{Arbitrary, Gen};

use crate::expr;
use crate::memory::{Memory, Register, simple as simple_mem};
use crate::module;
use crate::named::Named;
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
    SimpleMemPort(simple_mem::Port<Arc<Self>>),
    Instance(module::Instance),
}

impl Entity {
    /// Checks whether this entity can be declared via a [super::Statement]
    ///
    /// Returns true if the entity can be declared, which will be the case for
    /// most entities. Note that [module::Port]s cannot be declared.
    pub fn is_declarable(&self) -> bool {
        match self {
            Self::Port(..)  => false,
            _ => true,
        }
    }
}

impl From<Arc<module::Port>> for Entity {
    fn from(port: Arc<module::Port>) -> Self {
        Self::Port(port)
    }
}

impl From<Register<Arc<Entity>>> for Entity {
    fn from(register: Register<Arc<Entity>>) -> Self {
        Self::Register(register)
    }
}

impl From<Memory> for Entity {
    fn from(mem: Memory) -> Self {
        Self::Memory(mem)
    }
}

impl From<simple_mem::Port<Arc<Entity>>> for Entity {
    fn from(port: simple_mem::Port<Arc<Entity>>) -> Self {
        Self::SimpleMemPort(port)
    }
}

impl From<module::Instance> for Entity {
    fn from(inst: module::Instance) -> Self {
        Self::Instance(inst)
    }
}

impl expr::Reference for Arc<Entity> {
    fn flow(&self) -> Option<expr::Flow> {
        match self.as_ref() {
            Entity::Port(port)          => port.flow(),
            Entity::Wire{..}            => Some(expr::Flow::Duplex),
            Entity::Register(reg)       => reg.flow(),
            Entity::Node{..}            => Some(expr::Flow::Source),
            Entity::Memory(mem)         => mem.flow(),
            Entity::SimpleMemPort(port) => port.flow(),
            Entity::Instance(inst)      => inst.flow(),
        }
    }
}

impl Named for Entity {
    type Name = Arc<str>;

    fn name(&self) -> &Self::Name {
        match self {
            Entity::Port(port)          => port.name(),
            Entity::Wire{name, ..}      => name,
            Entity::Register(reg)       => reg.name(),
            Entity::Node{name, ..}      => name,
            Entity::Memory(mem)         => mem.name(),
            Entity::SimpleMemPort(port) => port.name(),
            Entity::Instance(inst)      => inst.name(),
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
            Entity::SimpleMemPort(port) => port.r#type().map_err(|_| self.clone()),
            Entity::Instance(inst)      => inst.r#type().map_err(|_| self.clone()),
        }
    }
}

#[cfg(test)]
impl expr::tests::TypedRef for Arc<Entity> {
    fn with_type(r#type: types::Type, flow: expr::Flow, g: &mut Gen) -> Self {
        use crate::tests::Identifier;

        use expr::tests::{expr_with_type, source_flow};

        fn field_to_port(field: &types::BundleField) -> Arc<module::Port> {
            let dir = match field.orientation() {
                types::Orientation::Normal  => module::Direction::Output,
                types::Orientation::Flipped => module::Direction::Input,
            };
            Arc::new(module::Port::new(field.name().clone(), field.r#type().clone(), dir))
        }

        let mut opts: Vec<&dyn Fn(Identifier, types::Type, &mut Gen) -> Entity> = match flow {
            expr::Flow::Source => vec![
                &|n, t, _| Arc::new(module::Port::new(n.to_string(), t, module::Direction::Input)).into(),
                &|n, t, g| Entity::Node{name: n.into(), value: expr_with_type(t, source_flow(g), g)},
            ],
            expr::Flow::Sink => vec![
                &|n, t, _| Arc::new(module::Port::new(n.to_string(), t, module::Direction::Output)).into(),
            ],
            expr::Flow::Duplex => vec![
                &|n, t, _| Entity::Wire{name: n.into(), r#type: t},
                &|n, t, g| Register::new(n, t, expr_with_type(types::GroundType::Clock, source_flow(g), g))
                    .into(),
            ],
        };

        if let (types::Type::Bundle(_), expr::Flow::Source) = (&r#type, flow) {
            opts.push(&|n, t, g| {
                let m = module::Module::new(
                    Identifier::arbitrary(g).into(),
                    t.fields().unwrap().map(field_to_port),
                    Arbitrary::arbitrary(g),
                );
                module::Instance::new(n, Arc::new(m)).into()
            })
        }

        Arc::new(g.choose(opts.as_ref()).unwrap()(Identifier::arbitrary(g), r#type, g))
    }
}

#[cfg(test)]
impl Arbitrary for Entity {
    fn arbitrary(g: &mut Gen) -> Self {
        use crate::tests::Identifier;

        use expr::tests::{expr_with_type, source_flow};

        let opts: [&dyn Fn(&mut Gen) -> Entity; 7] = [
            &|g| Arc::new(module::Port::arbitrary(g)).into(),
            &|g| Entity::Wire{name: Identifier::arbitrary(g).into(), r#type: Arbitrary::arbitrary(g)},
            &|g| Register::arbitrary(g).into(),
            &|g| Entity::Node{
                name: Identifier::arbitrary(g).into(),
                value: expr_with_type(types::Type::arbitrary(g), source_flow(g), g)
            },
            &|g| Memory::arbitrary(g).into(),
            &|g| simple_mem::Port::arbitrary(g).into(),
            &|g| module::Instance::arbitrary(g).into(),
        ];

        g.choose(&opts).unwrap()(g)
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        use crate::tests::Identifier;

        match self {
            Self::Port(port)            => Box::new(port.shrink().map(Into::into)),
            Self::Wire{name, r#type}    => {
                let res = (Identifier::from(name.as_ref()), r#type.clone())
                    .shrink()
                    .map(|(n, r#type)| Self::Wire{name: n.into(), r#type});
                Box::new(res)
            },
            Self::Register(reg)         => Box::new(reg.shrink().map(Into::into)),
            Self::Node{name, value}     => {
                let v = value.clone();
                let res = Identifier::from(name.as_ref())
                    .shrink()
                    .map(move |n| Self::Node{name: n.into(), value: v.clone()});
                Box::new(res)
            },
            Self::Memory(mem)           => Box::new(mem.shrink().map(Into::into)),
            Self::SimpleMemPort(port)   => Box::new(port.shrink().map(Into::into)),
            Self::Instance(inst)        => Box::new(inst.shrink().map(Into::into)),
        }
    }
}

