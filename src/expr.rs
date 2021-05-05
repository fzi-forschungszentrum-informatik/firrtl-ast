//! Datatypes and utilities specific to expressions

mod parsers;
mod primitive;

#[cfg(test)]
mod tests;

use std::fmt;
use std::sync::Arc;

#[cfg(test)]
use quickcheck::{Arbitrary, Gen};

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

#[cfg(test)]
impl Arbitrary for Expression<Identifier> {
    fn arbitrary(g: &mut Gen) -> Self {
        let opts: [&dyn Fn(&mut Gen) -> Self; 9] = [
            &|g| Self::UIntLiteral{value: Arbitrary::arbitrary(g), width: Arbitrary::arbitrary(g)},
            &|g| Self::SIntLiteral{value: Arbitrary::arbitrary(g), width: Arbitrary::arbitrary(g)},
            &|g| Self::Reference(Arbitrary::arbitrary(g)),
            &|g| Self::SubField{
                base: Arbitrary::arbitrary(g),
                index: Identifier::arbitrary(g).to_string().into()
            },
            &|g| Self::SubIndex{base: Arbitrary::arbitrary(g), index: Arbitrary::arbitrary(g)},
            &|g| Self::SubAccess{base: Arbitrary::arbitrary(g), index: Arbitrary::arbitrary(g)},
            &|g| Self::Mux{
                sel: Arbitrary::arbitrary(g),
                a: Arbitrary::arbitrary(g),
                b: Arbitrary::arbitrary(g)
            },
            &|g| Self::ValidIf{sel: Arbitrary::arbitrary(g), value: Arbitrary::arbitrary(g)},
            &|g| Self::PrimitiveOp(Arbitrary::arbitrary(g)),
        ];
        if g.size() > 0 {
            g.choose(&opts).unwrap()(&mut Gen::new(g.size() / 2))
        } else {
            Self::UIntLiteral{value: 0, width: 0}
        }
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        use std::iter::once;

        let from_vec = |v: Vec<Arc<Self>>| Box::new(
            v.into_iter().flat_map(|i| once(i.clone()).chain(i.shrink())).map(|e| e.as_ref().clone())
        );
        let from_single = |e: &Arc<Self>| Box::new(
            once(e.clone()).chain(e.shrink()).map(|e| e.as_ref().clone())
        );

        match self {
            Self::Reference(reference)      => Box::new(reference.shrink().map(Self::Reference)),
            Self::SubField{base, ..}        => from_single(base),
            Self::SubIndex{base, ..}        => from_single(base),
            Self::SubAccess{base, index}    => from_vec(vec![base.clone(), index.clone()]),
            Self::Mux{sel, a, b}            => from_vec(vec![sel.clone(), a.clone(), b.clone()]),
            Self::ValidIf{sel, value}       => from_vec(vec![sel.clone(), value.clone()]),
            Self::PrimitiveOp(op)           => Box::new(from_vec(op.sub_exprs())),
            _                               => Box::new(std::iter::empty()),
        }
    }
}


/// A reference to a named entity
pub trait Reference {
    /// Retrieve the name of the referenced entity
    fn name(&self) -> &str;
}

#[cfg(test)]
impl Reference for Identifier {
    fn name(&self) -> &str {
        self.as_ref()
    }
}

