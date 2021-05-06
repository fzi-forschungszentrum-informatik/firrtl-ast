//! Types

pub mod parsers;

mod display;
mod oriented;
mod r#type;

#[cfg(test)]
mod tests;


use std::fmt;

#[cfg(test)]
use quickcheck::{Arbitrary, Gen};

pub use oriented::OrientedType;
pub use r#type::{BundleField, Type};


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



/// FIRRTL ground type
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum GroundType {
    /// Unsigned integer type with width
    UInt(BitWidth),
    /// Signed integer type with width
    SInt(BitWidth),
    /// Fixed point type, with width and negative exponent
    Fixed(BitWidth, Option<i16>),
    /// Clock type
    Clock,
    /// Analog signal with number of wires
    Analog(BitWidth),
}

impl TypeExt for GroundType {
    fn eq(&self, rhs: &Self) -> bool {
        match (self, rhs) {
            (Self::UInt(_),     Self::UInt(_))     => true,
            (Self::SInt(_),     Self::SInt(_))     => true,
            (Self::Fixed(_, _), Self::Fixed(_, _)) => true,
            (Self::Clock,       Self::Clock)       => true,
            (Self::Analog(_),   Self::Analog(_))   => true,
            _ => false
        }
    }

    #[inline(always)]
    fn is_passive(&self) -> bool {
        true
    }
}

impl fmt::Display for GroundType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use display::{PointOff, Width};
        match self {
            Self::UInt(w)     => write!(f, "UInt{}", Width::from(w)),
            Self::SInt(w)     => write!(f, "SInt{}", Width::from(w)),
            Self::Fixed(w, p) => write!(f, "Fixed{}{}", Width::from(w), PointOff::from(p)),
            Self::Clock       => write!(f, "Clock"),
            Self::Analog(w)   => write!(f, "Analog{}", Width::from(w)),
        }
    }
}

#[cfg(test)]
impl Arbitrary for GroundType {
    fn arbitrary(g: &mut Gen) -> Self {
        let opts: [&dyn Fn(&mut Gen) -> Self; 5] = [
            &|g| Self::UInt(Arbitrary::arbitrary(g)),
            &|g| Self::SInt(Arbitrary::arbitrary(g)),
            &|g| Self::Fixed(Arbitrary::arbitrary(g), Arbitrary::arbitrary(g)),
            &|_| Self::Clock,
            &|g| Self::Analog(Arbitrary::arbitrary(g)),
        ];
        g.choose(&opts).unwrap()(g)
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        match self {
            Self::UInt(w)     => Box::new(w.shrink().map(Self::UInt)),
            Self::SInt(w)     => Box::new(w.shrink().map(Self::SInt)),
            Self::Fixed(w, p) => {
                use std::iter::once;
                let p = *p;
                Box::new(
                    once(*w).chain(w.shrink()).flat_map(move |w| p.shrink().map(move |p| Self::Fixed(w, p)))
                )
            },
            Self::Clock       => Box::new(std::iter::empty()),
            Self::Analog(w)   => Box::new(w.shrink().map(Self::Analog)),
        }
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

