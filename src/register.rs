//! Register type

use std::sync::Arc;

use crate::expr;
use crate::types;


/// Representation of a FIRRTL register
#[derive(Clone, Debug)]
pub struct Register<R: expr::Reference> {
    name: Arc<str>,
    r#type: types::Type,
    clock: expr::Expression<R>,
    reset: Option<(expr::Expression<R>, expr::Expression<R>)>,
}

impl<R: expr::Reference> Register<R> {
    /// Create a new register
    pub fn new(
        name: impl Into<Arc<str>>,
        r#type: impl Into<types::Type>,
        clock: impl Into<expr::Expression<R>>,
    ) -> Self {
        Self {name: name.into(), r#type: r#type.into(), clock: clock.into(), reset: Default::default()}
    }

    /// Retrieve the clock driving the register
    pub fn clock(&self) -> &expr::Expression<R> {
        &self.clock
    }

    /// Add a reset signal and value
    pub fn with_reset(
        self,
        signal: impl Into<expr::Expression<R>>,
        value: impl Into<expr::Expression<R>>
    ) -> Self {
        Self {reset: Some((signal.into(), value.into())), ..self}
    }

    /// Remove any reset signal and value
    pub fn without_reset(self) -> Self {
        Self {reset: None, ..self}
    }

    /// Retrieve the expression resetting the register
    pub fn reset_signal(&self) -> Option<&expr::Expression<R>> {
        self.reset.as_ref().map(|(sig, _)| sig)
    }

    /// Retrieve the expression the register is reset to
    pub fn reset_value(&self) -> Option<&expr::Expression<R>> {
        self.reset.as_ref().map(|(_, val)| val)
    }
}

impl<R: expr::Reference> expr::Reference for Register<R> {
    fn name(&self) -> &str {
        self.name.as_ref()
    }

    fn flow(&self) -> expr::Flow {
        expr::Flow::Duplex
    }
}

impl<R: expr::Reference> types::Typed for Register<R> {
    type Err = Self;

    type Type = types::Type;

    fn r#type(&self) -> Result<Self::Type, Self::Err> {
        Ok(self.r#type.clone())
    }
}

