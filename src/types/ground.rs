//! Ground type

use std::fmt;

#[cfg(test)]
use quickcheck::{Arbitrary, Gen};

use super::BitWidth;


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

impl GroundType {
    /// Retrieve the width of the ground type
    ///
    /// This function returns the width, i.e. the number of physical wires,
    /// corresponding to the type.
    pub fn width(&self) -> BitWidth {
        match self {
            Self::UInt(w)     => *w,
            Self::SInt(w)     => *w,
            Self::Fixed(w, _) => *w,
            Self::Clock       => Some(1),
            Self::Analog(w)   => *w,
        }
    }

    /// Create a copy of the type with the given width
    ///
    /// This function returns a copy of the type, with the width replaced by the
    /// given one. In the case of `Fixed`, the point will be preserved; in the
    /// case of `Clock`, this function will return a simple copy.
    pub fn with_width(&self, width: BitWidth) -> Self {
        match self {
            Self::UInt(_)     => Self::UInt(width),
            Self::SInt(_)     => Self::SInt(width),
            Self::Fixed(_, p) => Self::Fixed(width, *p),
            Self::Clock       => Self::Clock,
            Self::Analog(_)   => Self::Analog(width),
        }
    }
}

impl super::TypeExt for GroundType {
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

    #[inline(always)]
    fn ground_type(&self) -> Option<GroundType> {
        Some(self.clone())
    }
}

impl fmt::Display for GroundType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use super::display::{PointOff, Width};
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

