//! Types

pub mod combinator;
pub mod parsers;

mod display;
mod ground;
mod orientation;
mod oriented;
mod r#type;

#[cfg(test)]
mod tests;


pub use combinator::Combinator;
pub use ground::{GroundType, MaxWidth, combine_fixed_max};
pub use orientation::Orientation;
pub use oriented::OrientedType;
pub use r#type::{BundleField, Type};


/// Bit-width of a ground-type, i.e. the number of "physical" wires or signals
///
/// A bit-width may be undefined in some instances, i.e. they may need to be
/// inferred later. However, if it is defined, it can never be zero.
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
    /// In order to avoid confusion with `PartialEq` and `Eq`, users are encouraged
    /// to call `eq` as an associated function, e.g. as `TypeEq::eq(a, b)`.
    fn eq(&self, rhs: &Self) -> bool;

    /// Check whether the type is passive
    ///
    /// A type is passive if it contains no flipped sub-types or fields.
    fn is_passive(&self) -> bool;

    /// If the type refers to a ground type, return that ground type
    fn ground_type(&self) -> Option<GroundType>;
}

