//! Orientation

#[cfg(test)]
use quickcheck::{Arbitrary, Gen};


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

