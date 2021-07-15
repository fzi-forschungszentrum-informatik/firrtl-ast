//! Context for referencable entities used in statements

use std::sync::Arc;

use crate::memory::simple::Memory as SimpleMem;
use crate::module::Module;
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

