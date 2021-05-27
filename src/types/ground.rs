//! Ground type

use std::fmt;

#[cfg(test)]
use quickcheck::{Arbitrary, Gen};

use super::{BitWidth, Combinator};


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

/// Combinator impl for BitWidth combination of ground types
///
/// This `Combinator` combines `UInt`s, `SInt`s und `Analog` based on `BitWidth`
/// combination. `Clock`s are combined to a `Clock`. Combination of different
/// `GroundType` variants or `Fixed` will result in an `Err`.
impl<C: Combinator<BitWidth>> Combinator<GroundType> for C {
    fn combine<'a>(
        &self,
        lhs: &'a GroundType,
        rhs: &'a GroundType,
    ) -> Result<GroundType, (&'a GroundType, &'a GroundType)> {
        let combine = |l, r| self.combine(l, r).map_err(|_| (lhs, rhs));
        match (lhs, rhs) {
            (GroundType::UInt(l),   GroundType::UInt(r))   => combine(l, r).map(GroundType::UInt),
            (GroundType::SInt(l),   GroundType::SInt(r))   => combine(l, r).map(GroundType::SInt),
            (GroundType::Clock,     GroundType::Clock)     => Ok(GroundType::Clock),
            (GroundType::Analog(l), GroundType::Analog(r)) => combine(l, r).map(GroundType::Analog),
            _ => Err((lhs, rhs))
        }
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


/// Maximum width Combinator
///
/// Creating an `FnWidth` from `std::cmp::max` will yield a combinator which
/// selects the maximum width of the input `GroundType`s. However, it will yield
/// an error for fixed types.
///
/// This `Combinator` extends the `FnWidth` for fixed types. For two `Fixed`
/// variants, the combinator computes a fixed type, taking into account both
/// point offsets. All other combinations are forwarded to an `FnWidth`.
pub struct MaxWidth {}

impl MaxWidth {
    pub fn new() -> Self {
        Self {}
    }
}

impl Combinator<GroundType> for MaxWidth {
    fn combine<'a>(
        &self,
        lhs: &'a GroundType,
        rhs: &'a GroundType,
    ) -> Result<GroundType, (&'a GroundType, &'a GroundType)> {
        use std::cmp::max;

        use GroundType as GT;

        match (lhs, rhs) {
            (GT::Fixed(Some(lw), Some(lp)), GT::Fixed(Some(rw), Some(rp))) => Ok(
                GT::Fixed(combine_fixed_max((*lw, *lp), (*rw, *rp)), Some(max(*lp, *rp)))
            ),
            (GT::Fixed(..), GT::Fixed(..)) => Ok(GT::Fixed(None, None)),
            (l, r) => super::combinator::FnWidth::from(max).combine(l, r),
        }
    }
}


/// Compute the max width parameter for a combination of two fixed
///
/// This function effectively computes `max(lw - lp, rw - rp) + max(lp, rp)`
/// (cmp. section 10 of the FIRRTL spec), but tries to avoid underflow issues.
/// If the result doesn't fit into an `u16`, the function returns `None`.
pub fn combine_fixed_max(lhs: (u16, i16), rhs: (u16, i16)) -> BitWidth {
    use std::cmp::max;

    use std::convert::TryInto;

    let lw = lhs.0 as i32;
    let lp = lhs.1 as i32;
    let rw = rhs.0 as i32;
    let rp = rhs.1 as i32;
    (max(lw - lp, rw - rp) + max(lp, rp)).try_into().ok()
}

