// Copyright (c) 2021 FZI Forschungszentrum Informatik
// SPDX-License-Identifier: Apache-2.0
//! Ground type

use std::fmt;

#[cfg(test)]
use quickcheck::{Arbitrary, Gen};

use super::{BitWidth, Combinator, SBits, UBits};


/// FIRRTL ground type
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum GroundType {
    /// Unsigned integer type with width
    UInt(BitWidth),
    /// Signed integer type with width
    SInt(BitWidth),
    /// Fixed point type, with width and negative exponent
    Fixed(BitWidth, Option<SBits>),
    /// Clock type
    Clock,
    /// Reset type
    Reset(ResetKind),
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
            Self::Reset(_)    => Some(1),
            Self::Analog(w)   => *w,
        }
    }

    /// Create a copy of the type with the given width
    ///
    /// This function returns a copy of the type, with the width replaced by the
    /// given one. In the case of [GroundType::Fixed], the point will be
    /// preserved; in the case of [GroundType::Clock], this function will return
    /// a simple copy.
    pub fn with_width(&self, width: BitWidth) -> Self {
        match self {
            Self::UInt(_)     => Self::UInt(width),
            Self::SInt(_)     => Self::SInt(width),
            Self::Fixed(_, p) => Self::Fixed(width, *p),
            Self::Clock       => Self::Clock,
            Self::Reset(k)    => Self::Reset(*k),
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

/// [Combinator] impl for [BitWidth] combination of [GroundType]s
///
/// This [Combinator] combines [GroundType::UInt]s, [GroundType::SInt]s and
/// [GroundType::Analog] based on [BitWidth] combination. [GroundType::Clock]s
/// are combined to [GroundType::Clock]. Combination of different
/// [GroundType] variants or [GroundType::Fixed] will result in an `Err`.
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
        use ResetKind as R;

        match self {
            Self::UInt(w)           => write!(f, "UInt{}", Width::from(w)),
            Self::SInt(w)           => write!(f, "SInt{}", Width::from(w)),
            Self::Fixed(w, p)       => write!(f, "Fixed{}{}", Width::from(w), PointOff::from(p)),
            Self::Clock             => write!(f, "Clock"),
            Self::Reset(R::Regular) => write!(f, "Reset"),
            Self::Reset(R::Async)   => write!(f, "AsyncReset"),
            Self::Analog(w)         => write!(f, "Analog{}", Width::from(w)),
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
            Self::Fixed(w, p) => Box::new((*w, *p).shrink().map(|(w, p)| Self::Fixed(w, p))),
            Self::Analog(w)   => Box::new(w.shrink().map(Self::Analog)),
            _                 => Box::new(std::iter::empty()),
        }
    }
}


/// Kind of reset signal
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum ResetKind {Regular, Async}


/// Maximum width [Combinator]
///
/// Creating an [FnWidth][super::combinator::FnWidth] using [std::cmp::max] will
/// yield a [Combinator] which selects the maximum width of the input
/// [GroundType]s.  However, it will yield an error for fixed types.
///
/// This [Combinator] extends the [FnWidth][super::combinator::FnWidth] for
/// fixed types. For two [GroundType::Fixed], the [Combinator] computes a fixed
/// type, taking into account both point offsets. All other combinations are
/// forwarded to an [FnWidth][super::combinator::FnWidth].
pub struct MaxWidth {}

impl MaxWidth {
    pub fn new() -> Self {
        Self {}
    }

    /// Combine two widths
    pub fn combine_widths(lhs: BitWidth, rhs: BitWidth) -> BitWidth {
        if let (Some(l), Some(r)) = (lhs, rhs) {
            Some(std::cmp::max(l, r))
        } else {
            None
        }
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
            (l, r) => super::combinator::FnWidth::from(|l, r| Some(max(l, r))).combine(l, r),
        }
    }
}


/// Compute the max width parameter for a combination of two fixed
///
/// This function effectively computes `max(lw - lp, rw - rp) + max(lp, rp)`
/// (cmp. section 10 of the FIRRTL spec), but tries to avoid underflow issues.
/// If the result doesn't fit into an [UBits], the function returns `None`.
pub fn combine_fixed_max(lhs: (UBits, SBits), rhs: (UBits, SBits)) -> BitWidth {
    use std::cmp::max;

    use std::convert::TryInto;

    let lw: i32 = lhs.0.into();
    let lp: i32 = lhs.1.into();
    let rw: i32 = rhs.0.into();
    let rp: i32 = rhs.1.into();
    (max(lw - lp, rw - rp) + max(lp, rp)).try_into().ok()
}

