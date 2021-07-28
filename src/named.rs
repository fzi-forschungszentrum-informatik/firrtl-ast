// Copyright (c) 2021 FZI Forschungszentrum Informatik
// SPDX-License-Identifier: Apache-2.0
//! Common utility for named items


/// Named item
///
/// This trait allows accessing the item's name both in it's storage type and
/// a `&str`. The former will usually allow more efficient cloning of the name.
pub trait Named {
    /// Type used to store the name
    type Name: AsRef<str>;

    /// Retrieve a reference to the stored name
    fn name(&self) -> &Self::Name;

    /// Retrieve the item's name as a `&str`
    fn name_ref(&self) -> &str {
        self.name().as_ref()
    }
}

