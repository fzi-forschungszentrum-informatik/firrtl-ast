// Copyright (c) 2021 FZI Forschungszentrum Informatik
// SPDX-License-Identifier: Apache-2.0
//! FIRRTL types

pub mod combinator;
pub(crate) mod parsers;

mod display;
mod ground;
mod orientation;
mod oriented;
mod r#type;

#[cfg(test)]
mod tests;


pub use combinator::Combinator;
pub use ground::{GroundType, MaxWidth, ResetKind, combine_fixed_max};
pub use orientation::Orientation;
pub use oriented::OrientedType;
pub use r#type::{BundleField, Type};

#[cfg(test)]
pub use r#type::bundle_fields;


/// Bit-width of a [GroundType], i.e. the number of "physical" wires or signals
///
/// A bit-width may be undefined in some instances, i.e. they may need to be
/// inferred later.
pub type BitWidth = Option<u16>;

/// Number of elements in a vector
pub type VecWidth = u16;


/// Trait representing common FIRRTL type concepts
pub trait TypeExt {
    /// Check whether this type is type equivalent to another one
    ///
    /// The FIRRTL specification contains a definition of type equivalence. This
    /// function determines whether two types are equivalent under that
    /// definition.
    ///
    /// In order to avoid confusion with [PartialEq] and [Eq], users are encouraged
    /// to call `eq` as an associated function, e.g. as `TypeEq::eq(a, b)`.
    fn eq(&self, rhs: &Self) -> bool;

    /// Check whether the type is passive
    ///
    /// A type is passive if it contains no flipped sub-types or fields.
    fn is_passive(&self) -> bool;

    /// If the type refers to a ground type, return that ground type
    fn ground_type(&self) -> Option<GroundType>;
}


/// A typed entity
pub trait Typed: Sized {
    /// Error type
    type Err;

    /// Type of type for this class of entities
    type Type: TypeExt;

    /// Determine the base type of this entity
    ///
    /// This function determines the basic type of the entity, i.e. without
    /// any widths inferred. The type returned will not contain any width which
    /// may conflict with the inferred type for the entity.
    ///
    /// This function is not required to perform an exhaustive type-check.
    fn r#type(&self) -> Result<Self::Type, Self::Err>;
}


/// Compute the width necessary to address the given number of elements
pub fn required_address_width(num: impl Into<u128>) -> u16 {
    let mut res = 0;
    let mut num = num.into().saturating_sub(1);
    while num > 0 {
        num = num >> 1;
        res = res + 1;
    }
    res
}

