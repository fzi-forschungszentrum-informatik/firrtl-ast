// Copyright (c) 2021 FZI Forschungszentrum Informatik
// SPDX-License-Identifier: Apache-2.0
//! FIRRTL register

use std::fmt;
use std::sync::Arc;

#[cfg(test)]
use quickcheck::{Arbitrary, Gen};

use crate::expr;
use crate::named::Named;
use crate::types;


/// Representation of a FIRRTL register
#[derive(Clone, Debug, PartialEq)]
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

    /// Add a reset signal and value
    pub fn with_optional_reset(self, reset: Option<(expr::Expression<R>, expr::Expression<R>)>) -> Self {
        Self {reset, ..self}
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
    fn flow(&self) -> Option<expr::Flow> {
        Some(expr::Flow::Duplex)
    }
}

impl<R: expr::Reference> Named for Register<R> {
    type Name = Arc<str>;

    fn name(&self) -> &Self::Name {
        &self.name
    }
}

impl<R: expr::Reference> types::Typed for Register<R> {
    type Err = Self;

    type Type = types::Type;

    fn r#type(&self) -> Result<Self::Type, Self::Err> {
        Ok(self.r#type.clone())
    }
}

impl<R: expr::Reference> fmt::Display for Register<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "reg {}: {}, {}", self.name(), self.r#type, self.clock())?;
        if let Some((sig, val)) = self.reset.as_ref() {
            write!(f, " with: (reset => ({}, {}))", sig, val)?;
        }
        Ok(())
    }
}

#[cfg(test)]
impl<R: expr::tests::TypedRef + Clone + 'static> Arbitrary for Register<R> {
    fn arbitrary(g: &mut Gen) -> Self {
        use crate::tests::Identifier;
        use expr::tests::{expr_with_type, source_flow};

        let res = Self::new(
            Identifier::arbitrary(g),
            types::Type::arbitrary(g),
            expr_with_type(types::GroundType::Clock, source_flow(g), g),
        );

        if bool::arbitrary(g) {
            let val_type = res.r#type.clone();
            res.with_reset(
                expr_with_type(types::GroundType::UInt(Some(1)), source_flow(g), g),
                expr_with_type(val_type, source_flow(g), g))
        } else {
            res
        }
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        use types::{GroundType as GT};
        use expr::tests::TypedExpr;

        let res = (
            crate::tests::Identifier::from(self.name().as_ref()),
            TypedExpr {expr: self.clock.clone(), r#type: GT::Clock.into()},
        ).shrink()
            .filter(|(_, c)| c.r#type == GT::Clock)
            .map({
                let t = self.r#type.clone();
                let r = self.reset.clone();
                move |(n, c)| Self::new(n, t.clone(), c.expr).with_optional_reset(r.clone())
            })
            .chain(self.r#type.shrink().map({
                // When shrinking the type, it's unlikely that we'll find a
                // suitable sub-expression for the reset-value and generating
                // one is just plain wrong. We take the easy way out and just
                // leave out the reset all together.
                let n = self.name.clone();
                let c = self.clock.clone();
                move |t| Self::new(n.clone(), t, c.clone())
            }));

        if let Some((sig, val)) = self.reset.as_ref() {
            let r_shrink = (
                TypedExpr {expr: sig.clone(), r#type: GT::UInt(Some(1)).into()},
                TypedExpr {expr: val.clone(), r#type: self.r#type.clone()},
            ).shrink().filter(|(s, _)| s.r#type == GT::UInt(Some(1))).map({
                let n = self.name.clone();
                let c = self.clock.clone();

                move |(s, v)| Self::new(n.clone(), v.r#type, c.clone())
                    .with_optional_reset(Some((s.expr, v.expr)))
            });

            Box::new(res.chain(r_shrink).chain(std::iter::once(self.clone().without_reset())))
        } else {
            Box::new(res)
        }
    }
}

