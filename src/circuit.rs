//! Circuit specific definitions and functions

use std::sync::Arc;

use crate::module::Module;


/// FIRRTL circuit
///
/// A `Circuit` is the top level construct in FIRRTL. It contains an arbitrary
/// number of modules, one of which is defined as the "top" module.
#[derive(Clone, Debug)]
pub struct Circuit {
    modules: Vec<Arc<Module>>,
    top: Arc<Module>
}

impl Circuit {
    /// Create a new circuit
    pub fn new(top_module: Arc<Module>, modules: impl IntoIterator<Item = Arc<Module>>) -> Self {
        Self {top: top_module, modules: modules.into_iter().collect()}
    }

    /// Get an iterator over all modules
    pub fn modules(&self) -> impl Iterator<Item = &Arc<Module>> {
        self.modules.iter()
    }

    /// Get the top level module
    pub fn top_module(&self) -> &Arc<Module> {
        &self.top
    }
}
