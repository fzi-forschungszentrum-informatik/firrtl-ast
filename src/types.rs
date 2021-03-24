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

impl Type {
    /// Convert to an `OrientedType` with a defined root orientation
    pub fn with_orientation(&self, orientation: Orientation) -> OrientedType {
        match self {
            Self::GroundType(g) => OrientedType::GroundType(*g, orientation),
            Self::Vector(t, w)  => OrientedType::Vector(Box::new(t.with_orientation(orientation)), *w),
            Self::Bundle(v)     => OrientedType::Bundle(
                v.iter().map(|(n, t, o)| (n.clone(), t.with_orientation(*o + orientation))).collect()
            ),
        }
    }
}


/// Oriented type
///
/// In an oriented type, the orientation is attached to the leaf nodes, i.e. the
/// ground types, rather than fields in a bundle.
#[derive(Clone, Debug)]
pub enum OrientedType {
    GroundType(GroundType, Orientation),
    Vector(Box<OrientedType>, Width),
    Bundle(Vec<(String, OrientedType)>),
}

impl OrientedType {
    /// Clone this type with all orientations flipped
    pub fn flipped(&self) -> Self {
        match self {
            Self::GroundType(g, o) => Self::GroundType(*g, *o + Orientation::Flipped),
            Self::Vector(t, w)     => Self::Vector(Box::new(t.flipped()), *w),
            Self::Bundle(v)        => Self::Bundle(v.iter().map(|(n, t)| (n.clone(), t.flipped())).collect()),
        }
    }
}

impl From<&Type> for OrientedType {
    fn from(t: &Type) -> Self {
        t.with_orientation(Default::default())
    }
}

