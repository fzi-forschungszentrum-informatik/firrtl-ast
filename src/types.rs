//! Types


/// (Bit)width
pub type Width = u16;


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


/// FIRRTL ground type
#[derive(Copy, Clone, Debug)]
pub enum GroundType {
    UInt(Width),
    SInt(Width),
    Fixed(Width, i16),
    Clock,
    Analog(Width),
}

impl TypeEq for GroundType {
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

    /// Check whether this type is weakly equivalent to another type
    ///
    /// Two `Type`s are weakly equivalent if their corresponding `OrientedType`s
    /// are (type) equivalent.
    pub fn weak_eq(&self, rhs: &Self) -> bool {
        TypeEq::eq(&self.with_orientation(Default::default()), &rhs.with_orientation(Default::default()))
    }
}

impl TypeEq for Type {
    fn eq(&self, rhs: &Self) -> bool {
        match (self, rhs) {
            (Self::GroundType(t1), Self::GroundType(t2)) => TypeEq::eq(t1, t2),
            (Self::Vector(t1, w1), Self::Vector(t2, w2)) => TypeEq::eq(t1.as_ref(), t2.as_ref()) && w1 == w2,
            (Self::Bundle(v1), Self::Bundle(v2)) => if v1.len() == v2.len() {
                v1.iter()
                    .zip(v2.iter())
                    .all(|((n1, t1, o1), (n2, t2, o2))| n1 == n2 && TypeEq::eq(t1, t2) && o1 == o2)
            } else {
                false
            },
            _ => false
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

impl TypeEq for OrientedType {
    fn eq(&self, rhs: &Self) -> bool {
        match (self, rhs) {
            (Self::GroundType(t1, o1), Self::GroundType(t2, o2)) => TypeEq::eq(t1, t2) && o1 == o2,
            (Self::Vector(t1, w1), Self::Vector(t2, w2)) => TypeEq::eq(t1.as_ref(), t2.as_ref()) && w1 == w2,
            (Self::Bundle(v1), Self::Bundle(v2)) => if v1.len() == v2.len() {
                v1.iter().zip(v2.iter()).all(|((n1, t1), (n2, t2))| n1 == n2 && TypeEq::eq(t1, t2))
            } else {
                false
            },
            _ => false
        }
    }
}

impl From<&Type> for OrientedType {
    fn from(t: &Type) -> Self {
        t.with_orientation(Default::default())
    }
}


/// Trait representing the type equivalence concept in FIRRTL
///
/// The FIRRTL specification defines some specific rules for type equivalence.
/// Types implementing this trait express those rules via their implementation
/// of the `eq` function.
///
/// In order to avoid confusion with `PartialEq` and `Eq`, users are encouraged
/// to call `eq` as an associated function, e.g. as `TypeEq::eq(a, b)`.
pub trait TypeEq {
    /// Check whether this type is type equivalent to another one
    fn eq(&self, rhs: &Self) -> bool;
}

