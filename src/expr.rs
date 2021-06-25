//! Datatypes and utilities specific to expressions

pub(crate) mod parsers;
pub mod primitive;

#[cfg(test)]
pub mod tests;

use std::fmt;
use std::sync::Arc;

use crate::types;
use types::Typed;

#[cfg(test)]
use crate::tests::Identifier;


/// A FIRRTL expression
#[derive(Clone, Debug, PartialEq)]
pub enum Expression<R: Reference> {
    /// An UInt literal
    UIntLiteral{value: u128, width: u16},
    /// An SInt literal
    SIntLiteral{value: i128, width: u16},
    /// A referernce expression
    Reference(R),
    /// A sub-field expression
    SubField{base: Arc<Expression<R>>, index: Arc<str>},
    /// A sub-index expression
    SubIndex{base: Arc<Expression<R>>, index: u16},
    /// A sub-access expression
    SubAccess{base: Arc<Expression<R>>, index: Arc<Expression<R>>},
    /// A multiplexer expression
    Mux{sel: Arc<Expression<R>>, a: Arc<Expression<R>>, b: Arc<Expression<R>>},
    /// A valid-if expression
    ValidIf{sel: Arc<Expression<R>>, value: Arc<Expression<R>>},
    /// A primitive operation
    PrimitiveOp(primitive::Operation<R>),
}

impl<R> Expression<R>
where Self: Typed<Type = types::Type, Err = Expression<R>> + Clone,
      R: Reference,
{
    /// Determine the flow of this expression
    pub fn flow(&self) -> Result<Flow, <Self as types::Typed>::Err> {
        match self {
            Self::Reference(reference)  => Ok(reference.flow()),
            Self::SubField{base, index} => base.flow().and_then(|f| base
                .r#type()
                .and_then(|b| b.field(index.as_ref()).map(|b| f + b.orientation()).ok_or(self.clone()))
            ),
            Self::SubIndex{base, ..}    => base.flow(),
            Self::SubAccess{base, ..}   => base.flow(),
            _                           => Ok(Flow::Source),
        }
    }

    /// Retrieve all references in this expression
    pub fn references(&self) -> impl Iterator<Item = &R> {
        use transiter::AutoTransIter;

        // We use a depth-first search for discovering references, as a
        // breadth-first search would probably require more elements to be
        // buffered inside the iterator.
        self.trans_iter()
            .depth_first_unordered()
            .filter_map(|e| if let Self::Reference(r) = e { Some(r) } else { None })
    }
}

impl<R: Reference> From<R> for Expression<R> {
    fn from(reference: R) -> Self {
        Self::Reference(reference)
    }
}

impl<R: Reference> From<primitive::Operation<R>> for Expression<R> {
    fn from(op: primitive::Operation<R>) -> Self {
        Self::PrimitiveOp(op)
    }
}

impl<R> Typed for Expression<R>
    where R: Reference + Typed + Clone,
          R::Type: Into<types::Type>,
{
    type Err = Self;

    type Type = types::Type;

    fn r#type(&self) -> Result<Self::Type, Self::Err> {
        use types::{Combinator, GroundType as GT, MaxWidth};

        match self {
            Self::UIntLiteral{width, ..}    => Ok(GT::UInt(Some(*width)).into()),
            Self::SIntLiteral{width, ..}    => Ok(GT::SInt(Some(*width)).into()),
            Self::Reference(reference)      => reference.r#type().map(Into::into).map_err(|_| self.clone()),
            Self::SubField{base, index}     => base
                .r#type()
                .and_then(|t| t.field(index.as_ref()).map(|f| f.r#type().clone()).ok_or(self.clone())),
            Self::SubIndex{base, ..}        => base
                .r#type()
                .and_then(|t| t.vector_base().map(|b| b.as_ref().clone()).ok_or(self.clone())),
            Self::SubAccess{base, ..}       => base
                .r#type()
                .and_then(|t| t.vector_base().map(|b| b.as_ref().clone()).ok_or(self.clone())),
            Self::Mux{a, b, ..}             => MaxWidth::new()
                .combine(&a.r#type()?, &b.r#type()?)
                .map_err(|_| self.clone()),
            Self::ValidIf{value, ..}        => value.r#type(),
            Self::PrimitiveOp(op)           => op.r#type().map(Into::into).map_err(|_| self.clone()),
        }
    }
}

impl<'a, R: Reference> transiter::AutoTransIter<&'a Expression<R>> for &'a Expression<R> {
    type RecIter = Vec<Self>;

    fn recurse(item: &Self) -> Self::RecIter {
        match item {
            Expression::SubField{base, ..}        => vec![base.as_ref()],
            Expression::SubIndex{base, ..}        => vec![base.as_ref()],
            Expression::SubAccess{base, index}    => vec![base.as_ref(), index.as_ref()],
            Expression::Mux{sel, a, b}            => vec![sel.as_ref(), a.as_ref(), b.as_ref()],
            Expression::ValidIf{sel, value}       => vec![sel.as_ref(), value.as_ref()],
            Expression::PrimitiveOp(op)           => op.sub_exprs().into_iter().map(AsRef::as_ref).collect(),
            _ => Default::default(),
        }
    }
}

impl<R: Reference> fmt::Display for Expression<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UIntLiteral{value, width} => write!(f, "UInt<{}>({})", width, value),
            Self::SIntLiteral{value, width} => write!(f, "SInt<{}>({})", width, value),
            Self::Reference(reference)      => fmt::Display::fmt(reference.name(), f),
            Self::SubField{base, index}     => write!(f, "{}.{}", base, index),
            Self::SubIndex{base, index}     => write!(f, "{}[{}]", base, index),
            Self::SubAccess{base, index}    => write!(f, "{}[{}]", base, index),
            Self::Mux{sel, a, b}            => write!(f, "mux({}, {}, {})", sel, a, b),
            Self::ValidIf{sel, value}       => write!(f, "validif({}, {})", sel, value),
            Self::PrimitiveOp(op)           => fmt::Display::fmt(op, f),
        }
    }
}


/// A reference to a named entity
pub trait Reference {
    /// Retrieve the name of the referenced entity
    fn name(&self) -> &str;

    /// Retrieve the flow associated with the referenced entity
    fn flow(&self) -> Flow;
}

#[cfg(test)]
impl Reference for Identifier {
    fn name(&self) -> &str {
        self.as_ref()
    }

    fn flow(&self) -> Flow {
        Flow::Duplex
    }
}


/// Possible data flow
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Flow {
    Source,
    Sink,
    Duplex,
}

impl Flow {
    /// Determine whether the flow allows an entity to serve as a source of data
    ///
    /// This function returns true if the flow is either `Source` or `Duplex`.
    pub fn is_source(&self) -> bool {
        match self {
            Self::Source => true,
            Self::Sink   => false,
            Self::Duplex => true,
        }
    }

    /// Determine whether the flow allows an entity to serve as a fink for data
    ///
    /// This function returns true if the flow is either `Sink` or `Duplex`.
    pub fn is_sink(&self) -> bool {
        match self {
            Self::Source => false,
            Self::Sink   => true,
            Self::Duplex => true,
        }
    }
}

impl std::ops::Add<types::Orientation> for Flow {
    type Output = Self;

    fn add(self, rhs: types::Orientation) -> Self::Output {
        use types::Orientation as O;

        match (self, rhs) {
            (v,            O::Normal ) => v,
            (Self::Source, O::Flipped) => Self::Sink,
            (Self::Sink,   O::Flipped) => Self::Source,
            (Self::Duplex, O::Flipped) => Self::Duplex,
        }
    }
}

#[cfg(test)]
impl quickcheck::Arbitrary for Flow {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        g.choose(&[Self::Source, Self::Sink, Self::Duplex]).unwrap().clone()
    }
}

