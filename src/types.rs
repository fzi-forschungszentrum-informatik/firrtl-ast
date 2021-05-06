//! Types

pub mod parsers;

mod display;
mod ground;
mod oriented;
mod r#type;

#[cfg(test)]
mod tests;


#[cfg(test)]
use quickcheck::{Arbitrary, Gen};

pub use oriented::OrientedType;
pub use r#type::{BundleField, Type};
pub use ground::GroundType;


/// Bit-width of a ground-type, i.e. the number of "physical" wires or signals
///
/// A bit-width may be undefined in some instances, i.e. they may need to be
/// inferred later. However, if it is defined, it can never be zero.
pub type BitWidth = Option<u16>;

/// Number of elements in a vector
pub type VecWidth = u16;


/// Orientation
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Orientation {
    Normal,
    Flipped
}

impl Default for Orientation {
    fn default() -> Self {
        Self::Normal
    }
}

impl std::ops::Add for Orientation {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Normal,  Self::Normal)  => Self::Normal,
            (Self::Normal,  Self::Flipped) => Self::Flipped,
            (Self::Flipped, Self::Normal)  => Self::Flipped,
            (Self::Flipped, Self::Flipped) => Self::Normal,
        }
    }
}

#[cfg(test)]
impl Arbitrary for Orientation {
    fn arbitrary(g: &mut Gen) -> Self {
        *g.choose(&[Self::Normal, Self::Flipped]).unwrap()
    }
}


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
}

