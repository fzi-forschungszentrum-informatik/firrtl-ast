//! FIRRTL Type

use std::fmt;
use std::sync::Arc;

#[cfg(test)]
use quickcheck::{Arbitrary, Gen};

use super::{Combinator, GroundType, Orientation, OrientedType, TypeExt};

/// FIRRTL Type
#[derive(Clone, PartialEq, Debug)]
pub enum Type {
    GroundType(GroundType),
    Vector(Arc<Self>, super::VecWidth),
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

    /// If this type is a vector type, return the base type
    ///
    /// This function returns the type of a vector element or `None`, if called
    /// on a type not a vector type.
    pub fn vector_base(&self) -> Option<&Arc<Self>> {
        if let Self::Vector(t, _) = self {
            Some(t)
        } else {
            None
        }
    }

    /// Return the bundle field with the given name
    ///
    /// If the type is not a bundle type or the bundle does not contain a field
    /// with the given name, this function returns `None`.
    pub fn field(&self, field: &str) -> Option<&BundleField> {
        if let Self::Bundle(v) = self {
            v.iter().find(|f| f.name().as_ref() == field)
        } else {
            None
        }
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

    fn is_passive(&self) -> bool {
        match self {
            Self::GroundType(t) => t.is_passive(),
            Self::Vector(t, _) => t.is_passive(),
            Self::Bundle(v) => v
                .iter()
                .all(|f| f.orientation() == Orientation::Normal && f.r#type().is_passive()),
        }
    }

    fn ground_type(&self) -> Option<GroundType> {
        if let Self::GroundType(g) = self {
            Some(*g)
        } else {
            None
        }
    }
}

impl<C: Combinator<GroundType>> Combinator<Type> for C {
    fn combine<'a>(&self, lhs: &'a Type, rhs: &'a Type) -> Result<Type, (&'a Type, &'a Type)> {
        match (lhs, rhs) {
            (Type::GroundType(t1), Type::GroundType(t2)) => self.combine(t1, t2)
                .map_err(|_| (lhs, rhs))
                .map(Into::into),
            (Type::Vector(t1, w1), Type::Vector(t2, w2)) if w1 == w2 =>
                <Self as Combinator<Type>>::combine(self, t1.as_ref(), t2.as_ref())
                    .map(|t| Type::Vector(Arc::new(t), *w1)),
            (Type::Bundle(v1), Type::Bundle(v2)) if v1.len() == v2.len() => {
                let mut res: Vec<_> = Default::default();
               v1.iter().zip(v2.iter()).try_for_each(|(f1, f2)|
                    <Self as Combinator<BundleField>>::combine(self, f1, f2).map(|f| res.push(f))
                ).map_err(|_| (lhs, rhs))?;
                Ok(res.into())
            },
            _ => Err((lhs, rhs))
        }.map(|res| if res == *lhs {
            lhs.clone()
        } else if res == *rhs {
            rhs.clone()
        } else {
            res
        })
    }
}

impl From<GroundType> for Type {
    fn from(g: GroundType) -> Self {
        Self::GroundType(g)
    }
}

impl From<Vec<BundleField>> for Type {
    fn from(v: Vec<BundleField>) -> Self {
        Self::Bundle(v.into())
    }
}

#[cfg(test)]
impl From<std::collections::HashMap<Arc<str>, BundleField>> for Type {
    fn from(v: std::collections::HashMap<Arc<str>, BundleField>) -> Self {
        Self::Bundle(v.into_iter().map(|(_, f)| f).collect())
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
            &|g| bundle_fields(u8::arbitrary(g).saturating_add(1) as usize, g).into(),
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

impl<C: Combinator<Type>> Combinator<BundleField> for C {
    fn combine<'a>(
        &self,
        lhs: &'a BundleField,
        rhs: &'a BundleField,
    ) -> Result<BundleField, (&'a BundleField, &'a BundleField)> {
        if lhs.name() == rhs.name() && lhs.orientation() == rhs.orientation() {
            <Self as Combinator<Type>>::combine(self, lhs.r#type(), rhs.r#type())
                .map(|t| BundleField::new(lhs.name().clone(), t, rhs.orientation()))
                .map_err(|_| (lhs, rhs))
        } else {
            Err((lhs, rhs))
        }
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

        Self::new(Identifier::arbitrary(g).into(), Arbitrary::arbitrary(g), Arbitrary::arbitrary(g))
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        let n = self.name.clone();
        let o = self.orientation;
        Box::new(self.r#type.shrink().map(move |t| Self::new(n.clone(), t, o)))
    }
}


/// Generate a hashmap containing `BundleField`s, mapped to by their name
///
/// Naturally, the `BundleField`s are guranteed to have unique names.
#[cfg(test)]
pub fn bundle_fields(max_size: usize, g: &mut Gen) -> std::collections::HashMap<Arc<str>, BundleField> {
    if max_size == 0 {
        Default::default()
    } else {
        let mut g = Gen::new(g.size() / max_size);
        (0..max_size)
            .map(|_| Arbitrary::arbitrary(&mut g))
            .map(|f: BundleField| (f.name().clone(), f))
            .collect()
    }
}

