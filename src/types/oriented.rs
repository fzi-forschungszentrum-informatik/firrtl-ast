//! Oriented type

use std::sync::Arc;

use super::{Orientation, TypeExt};


/// Oriented type
///
/// In an oriented type, the orientation is attached to the leaf nodes, i.e. the
/// ground types, rather than fields in a bundle.
#[derive(Clone, PartialEq, Debug)]
pub enum OrientedType {
    GroundType(super::GroundType, Orientation),
    Vector(Arc<Self>, super::VecWidth),
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

    fn is_passive(&self) -> bool {
        match self {
            Self::GroundType(t, o) => t.is_passive() && *o == Orientation::Normal,
            Self::Vector(t, _) => t.is_passive(),
            Self::Bundle(v) => v.iter().all(|(_, t)| t.is_passive()),
        }
    }

    fn ground_type(&self) -> Option<super::GroundType> {
        if let Self::GroundType(g, _) = self {
            Some(*g)
        } else {
            None
        }
    }
}

impl From<&super::Type> for OrientedType {
    fn from(t: &super::Type) -> Self {
        t.with_orientation(Default::default())
    }
}

