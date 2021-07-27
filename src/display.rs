// Copyright (c) 2021 FZI Forschungszentrum Informatik
// SPDX-License-Identifier: Apache-2.0
//! Genderal display utilities

use std::fmt;
use std::sync::Arc;


/// Utility for formatting comma separated lists
pub struct CommaSeparated<I, E>
where I: IntoIterator<Item = E> + Clone,
      E: fmt::Display,
{
    inner: I,
    with_preceding: bool,
}

impl<I, E> CommaSeparated<I, E>
where I: IntoIterator<Item = E> + Clone,
      E: fmt::Display,
{
    /// Include a preceding comma
    pub fn with_preceding(self) -> Self {
        Self {with_preceding: true, ..self}
    }
}

impl<'a, E: fmt::Display> From<&'a Arc<[E]>> for CommaSeparated<&'a [E], &'a E> {
    fn from(inner: &'a Arc<[E]>) -> Self {
        Self{inner: inner.as_ref(), with_preceding: false}
    }
}

impl<I, E> From<I> for CommaSeparated<I, E>
where I: IntoIterator<Item = E> + Clone,
      E: fmt::Display,
{
    fn from(inner: I) -> Self {
        Self{inner, with_preceding: false}
    }
}

impl<I, E> fmt::Display for CommaSeparated<I, E>
where I: IntoIterator<Item = E> + Clone,
      E: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut items = self.inner.clone().into_iter();
        if !self.with_preceding {
            items.next().map(|item| item.fmt(f)).transpose().map(|_| ())?
        }
        items.try_for_each(|item| write!(f, ", {}", item))
    }
}

