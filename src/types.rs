//! Types


/// (Bit)width
pub type Width = u16;


/// Orientation
#[derive(Copy, Clone, Debug)]
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


/// FIRRTL ground type
#[derive(Copy, Clone, Debug)]
pub enum GroundType {
    UInt(Width),
    SInt(Width),
    Fixed(Width, i16),
    Clock,
    Analog(Width),
}


/// FIRRTL Type
#[derive(Clone, Debug)]
pub enum Type {
    GroundType(GroundType),
    Vector(Box<Type>, Width),
    Bundle(Vec<(String, Type, Orientation)>),
}

