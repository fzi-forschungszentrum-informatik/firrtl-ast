//! Types

pub mod parsers;

mod display;

#[cfg(test)]
mod tests;


use std::fmt;
use std::sync::Arc;

#[cfg(test)]
use quickcheck::{Arbitrary, Gen};


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


/// FIRRTL Type
#[derive(Clone, PartialEq, Debug)]
pub enum Type {
    GroundType(GroundType),
    Vector(Arc<Self>, VecWidth),
    Bundle(Arc<[BundleField]>),
}

impl Type {
    /// Convert to an `OrientedType` with a defined root orientation
    pub fn with_orientation(&self, orientation: Orientation) -> OrientedType {
        match self {
            Self::GroundType(g) => OrientedType::GroundType(*g, orientation),
            Self::Vector(t, w)  => OrientedType::Vector(Arc::new(t.with_orientation(orientation)), *w),
            Self::Bundle(v)     => OrientedType::Bundle(
                v.iter()
                    .map(|f| (f.name().clone(), f.r#type().with_orientation(f.orientation() + orientation)))
                    .collect()
            ),
        }
    }

    /// Check whether this type is weakly equivalent to another type
    ///
    /// Two `Type`s are weakly equivalent if their corresponding `OrientedType`s
    /// are (type) equivalent.
    pub fn weak_eq(&self, rhs: &Self) -> bool {
        TypeExt::eq(&self.with_orientation(Default::default()), &rhs.with_orientation(Default::default()))
    }
}

impl TypeExt for Type {
    fn eq(&self, rhs: &Self) -> bool {
        match (self, rhs) {
            (Self::GroundType(t1), Self::GroundType(t2)) => TypeExt::eq(t1, t2),
            (Self::Vector(t1, w1), Self::Vector(t2, w2)) => TypeExt::eq(t1.as_ref(), t2.as_ref()) && w1 == w2,
            (Self::Bundle(v1), Self::Bundle(v2)) => if v1.len() == v2.len() {
                v1.iter()
                    .zip(v2.iter())
                    .all(|(f1, f2)| f1.name() == f2.name() &&
                        TypeExt::eq(f1.r#type(), f2.r#type()) &&
                        f1.orientation() == f1.orientation())
            } else {
                false
            },
            _ => false
        }
    }
}

impl From<GroundType> for Type {
    fn from(g: GroundType) -> Self {
        Self::GroundType(g)
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GroundType(g) => fmt::Display::fmt(g, f),
            Self::Vector(t, w)  => write!(f, "{}[{}]", t, w),
            Self::Bundle(v)     => {
                let mut fields = v.iter();
                write!(f, "{{")?;
                fields.next().map(|field| fmt::Display::fmt(&field, f)).transpose().map(|_| ())?;
                fields.try_for_each(|field| write!(f, ", {}", field))?;
                write!(f, "}}")
            },
        }
    }
}

#[cfg(test)]
impl Arbitrary for Type {
    fn arbitrary(g: &mut Gen) -> Self {
        let opts: [&dyn Fn(&mut Gen) -> Self; 3] = [
            &|g| Self::GroundType(Arbitrary::arbitrary(g)),
            &|g| Self::Vector(Arbitrary::arbitrary(g), Arbitrary::arbitrary(g)),
            &|g| {
                let len = u8::arbitrary(g).saturating_add(1);
                let mut g = Gen::new(g.size() / len as usize);
                Self::Bundle((0..len).map(|_| Arbitrary::arbitrary(&mut g)).collect())
            },
        ];
        if g.size() > 0 {
            g.choose(&opts).unwrap()(g)
        } else {
            Self::GroundType(Arbitrary::arbitrary(g))
        }
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        match self {
            Self::GroundType(g) => Box::new(g.shrink().map(Self::GroundType)),
            Self::Vector(t, w) => {
                use std::iter::once;
                let w = *w;
                let res = once(t.clone())
                    .chain(t.shrink())
                    .flat_map(move |t| w.shrink().map(move |w| Self::Vector(t.clone(), w)));
                Box::new(res)
            },
            Self::Bundle(v) => Box::new(
                v.to_vec().shrink().filter(move |v| !v.is_empty()).map(Into::into).map(Self::Bundle)
            )
        }
    }
}


/// A field in a bundle
#[derive(Clone, PartialEq, Debug)]
pub struct BundleField {
    name: Arc<str>,
    r#type: Type,
    orientation: Orientation,
}

impl BundleField {
    /// Create a new field with the given name, type and orientation
    pub fn new(name: Arc<str>, r#type: Type, orientation: Orientation) -> Self {
        Self {name, r#type, orientation}
    }

    /// Retrieve the field's name
    pub fn name(&self) -> &Arc<str> {
        &self.name
    }

    /// Retrieve the field's type
    pub fn r#type(&self) -> &Type {
        &self.r#type
    }

    /// Retrieve the field's orientation
    pub fn orientation(&self) -> Orientation {
        self.orientation
    }
}

impl fmt::Display for BundleField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.orientation == Orientation::Flipped {
            write!(f, "flip ")?;
        }

        write!(f, "{}: {}", self.name, self.r#type)
    }
}

#[cfg(test)]
impl Arbitrary for BundleField {
    fn arbitrary(g: &mut Gen) -> Self {
        use crate::tests::Identifier;

        Self::new(
            Identifier::arbitrary(g).to_string().into(),
            Arbitrary::arbitrary(g),
            Arbitrary::arbitrary(g)
        )
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        let n = self.name.clone();
        let o = self.orientation;
        Box::new(self.r#type.shrink().map(move |t| Self::new(n.clone(), t, o)))
    }
}




/// Oriented type
///
/// In an oriented type, the orientation is attached to the leaf nodes, i.e. the
/// ground types, rather than fields in a bundle.
#[derive(Clone, PartialEq, Debug)]
pub enum OrientedType {
    GroundType(GroundType, Orientation),
    Vector(Arc<Self>, VecWidth),
    Bundle(Arc<[(Arc<str>, Self)]>),
}

impl OrientedType {
    /// Clone this type with all orientations flipped
    pub fn flipped(&self) -> Self {
        match self {
            Self::GroundType(g, o) => Self::GroundType(*g, *o + Orientation::Flipped),
            Self::Vector(t, w)     => Self::Vector(Arc::new(t.flipped()), *w),
            Self::Bundle(v)        => Self::Bundle(v.iter().map(|(n, t)| (n.clone(), t.flipped())).collect()),
        }
    }
}

impl TypeExt for OrientedType {
    fn eq(&self, rhs: &Self) -> bool {
        match (self, rhs) {
            (Self::GroundType(t1, o1), Self::GroundType(t2, o2)) => TypeExt::eq(t1, t2) && o1 == o2,
            (Self::Vector(t1, w1), Self::Vector(t2, w2)) => TypeExt::eq(t1.as_ref(), t2.as_ref()) && w1 == w2,
            (Self::Bundle(v1), Self::Bundle(v2)) => if v1.len() == v2.len() {
                v1.iter().zip(v2.iter()).all(|((n1, t1), (n2, t2))| n1 == n2 && TypeExt::eq(t1, t2))
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
}

