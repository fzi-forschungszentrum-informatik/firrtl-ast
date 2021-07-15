//! Context for referencable entities used in statements

use std::collections::HashMap;
use std::sync::Arc;

use crate::memory::simple::Memory as SimpleMem;
use crate::module::{Module, Port as ModPort};
use super::entity::Entity;


/// Context for named things
///
/// A `Context` allows looking up `Entity`s, `SimpleMem`s and `Module`s.
pub trait Context {
    /// Retrieve the entity with the given name, if any
    fn entity(&self, name: &str) -> Option<Arc<Entity>>;

    /// Add an entity
    fn add_entity(&mut self, entity: Arc<Entity>);

    /// Retrieve the memory with the given name, if any
    fn memory(&self, name: &str) -> Option<Arc<SimpleMem>>;

    /// Add a simple memory
    fn add_memory(&mut self, memory: Arc<SimpleMem>);

    /// Retrieve the module with the given name
    fn module(&self, name: &str) -> Option<Arc<Module>>;
}


/// Toplevel Context
#[derive(Clone, Debug, Default)]
pub struct TopContext<M> {
    entities: HashMap<Arc<str>, Arc<Entity>>,
    memories: HashMap<Arc<str>, Arc<SimpleMem>>,
    module: M,
}

impl<M> TopContext<M> {
    /// Create a new toplevel Context
    pub fn new(module: M) -> Self {
        Self {entities: Default::default(), memories: Default::default(), module}
    }

    /// Create a new toplevel Context
    pub fn with_entities(self, entities: impl IntoIterator<Item = Arc<Entity>>) -> Self {
        use crate::expr::Reference;

        Self {entities: entities.into_iter().map(|e| (e.name().into(), e)).collect(), ..self}
    }

    /// Create a new toplevel Context
    pub fn with_ports(self, ports: impl IntoIterator<Item = Arc<ModPort>>) -> Self {
        self.with_entities(ports.into_iter().map(Into::into).map(Arc::new))
    }
}

impl<M> From<M> for TopContext<M> {
    fn from(module: M) -> Self {
        Self::new(module)
    }
}

impl<M: Fn(&str) -> Option<Arc<Module>>> Context for TopContext<M> {
    fn entity(&self, name: &str) -> Option<Arc<Entity>> {
        self.entities.get(name).cloned()
    }

    fn add_entity(&mut self, entity: Arc<Entity>) {
        use crate::expr::Reference;

        self.entities.insert(entity.name().into(), entity);
    }

    fn memory(&self, name: &str) -> Option<Arc<SimpleMem>> {
        self.memories.get(name).cloned()
    }

    fn add_memory(&mut self, memory: Arc<SimpleMem>) {
        self.memories.insert(memory.name().clone(), memory);
    }

    fn module(&self, name: &str) -> Option<Arc<Module>> {
        (self.module)(name)
    }
}

