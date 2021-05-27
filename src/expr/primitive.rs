//! Primitive operations

use std::fmt;
use std::num::NonZeroU16;
use std::sync::Arc;

#[cfg(test)]
use quickcheck::{Arbitrary, Gen};

use crate::types;

use super::{Expression, Reference};


/// A single ("primitive") operation
#[derive(Clone, Debug, PartialEq)]
pub enum Operation<R: Reference> {
    /// Arithmetic addition
    Add(Arc<Expression<R>>, Arc<Expression<R>>),
    /// Arithmetic substraction
    Sub(Arc<Expression<R>>, Arc<Expression<R>>),
    /// Arithmetic multiplication
    Mul(Arc<Expression<R>>, Arc<Expression<R>>),
    /// Arithmetic division
    Div(Arc<Expression<R>>, Arc<Expression<R>>),
    /// Arithmetic modulo operation
    Rem(Arc<Expression<R>>, Arc<Expression<R>>),
    /// Lower-than
    Lt(Arc<Expression<R>>, Arc<Expression<R>>),
    /// Lower or equal
    LEq(Arc<Expression<R>>, Arc<Expression<R>>),
    /// Greater-than
    Gt(Arc<Expression<R>>, Arc<Expression<R>>),
    /// Greater or equal
    GEq(Arc<Expression<R>>, Arc<Expression<R>>),
    /// Equal
    Eq(Arc<Expression<R>>, Arc<Expression<R>>),
    /// Not equal
    NEq(Arc<Expression<R>>, Arc<Expression<R>>),
    /// Padding
    Pad(Arc<Expression<R>>, NonZeroU16),
    /// Type cast
    Cast(Arc<Expression<R>>, types::GroundType),
    /// Shift left (static)
    Shl(Arc<Expression<R>>, NonZeroU16),
    /// Shift right (static)
    Shr(Arc<Expression<R>>, NonZeroU16),
    /// Shift left (dynamic)
    DShl(Arc<Expression<R>>, Arc<Expression<R>>),
    /// Shift right (dynamic)
    DShr(Arc<Expression<R>>, Arc<Expression<R>>),
    /// Arithmetic "convert to signed"
    Cvt(Arc<Expression<R>>),
    /// Arithmetic complement/negation
    Neg(Arc<Expression<R>>),
    /// Bitwise complement
    Not(Arc<Expression<R>>),
    /// Bitwise AND
    And(Arc<Expression<R>>, Arc<Expression<R>>),
    /// Bitwise OR
    Or(Arc<Expression<R>>, Arc<Expression<R>>),
    /// Bitwise XOR
    Xor(Arc<Expression<R>>, Arc<Expression<R>>),
    /// AND reduction to single bit
    AndReduce(Arc<Expression<R>>),
    /// OR reduction to single bit
    OrReduce(Arc<Expression<R>>),
    /// XOR reduction to single bit
    XorReduce(Arc<Expression<R>>),
    /// Concatenation
    Cat(Arc<Expression<R>>, Arc<Expression<R>>),
    /// Bit extraction
    Bits(Arc<Expression<R>>, Option<NonZeroU16>, Option<NonZeroU16>),
    /// Increase precision (of "fixed")
    IncPrecision(Arc<Expression<R>>, NonZeroU16),
    /// Decrease precision (of "fixed")
    DecPrecision(Arc<Expression<R>>, NonZeroU16),
    /// Set precision (of "fixed")
    SetPrecision(Arc<Expression<R>>, i16),
}

impl<R: Reference> Operation<R> {
    /// Retrieve all subexpressions used in the operation
    ///
    pub fn sub_exprs(&self) -> Vec<Arc<Expression<R>>> {
        match self {
            Self::Add(lhs, rhs)         => vec![lhs.clone(), rhs.clone()],
            Self::Sub(lhs, rhs)         => vec![lhs.clone(), rhs.clone()],
            Self::Mul(lhs, rhs)         => vec![lhs.clone(), rhs.clone()],
            Self::Div(lhs, rhs)         => vec![lhs.clone(), rhs.clone()],
            Self::Rem(lhs, rhs)         => vec![lhs.clone(), rhs.clone()],
            Self::Lt(lhs, rhs)          => vec![lhs.clone(), rhs.clone()],
            Self::LEq(lhs, rhs)         => vec![lhs.clone(), rhs.clone()],
            Self::Gt(lhs, rhs)          => vec![lhs.clone(), rhs.clone()],
            Self::GEq(lhs, rhs)         => vec![lhs.clone(), rhs.clone()],
            Self::Eq(lhs, rhs)          => vec![lhs.clone(), rhs.clone()],
            Self::NEq(lhs, rhs)         => vec![lhs.clone(), rhs.clone()],
            Self::Pad(sub, ..)          => vec![sub.clone()],
            Self::Cast(sub, ..)         => vec![sub.clone()],
            Self::Shl(sub, ..)          => vec![sub.clone()],
            Self::Shr(sub, ..)          => vec![sub.clone()],
            Self::DShl(sub, index)      => vec![sub.clone(), index.clone()],
            Self::DShr(sub, index)      => vec![sub.clone(), index.clone()],
            Self::Cvt(sub)              => vec![sub.clone()],
            Self::Neg(sub)              => vec![sub.clone()],
            Self::Not(sub)              => vec![sub.clone()],
            Self::And(lhs, rhs)         => vec![lhs.clone(), rhs.clone()],
            Self::Or(lhs, rhs)          => vec![lhs.clone(), rhs.clone()],
            Self::Xor(lhs, rhs)         => vec![lhs.clone(), rhs.clone()],
            Self::AndReduce(sub)        => vec![sub.clone()],
            Self::OrReduce(sub)         => vec![sub.clone()],
            Self::XorReduce(sub)        => vec![sub.clone()],
            Self::Cat(lhs, rhs)         => vec![lhs.clone(), rhs.clone()],
            Self::Bits(sub, ..)         => vec![sub.clone()],
            Self::IncPrecision(sub, ..) => vec![sub.clone()],
            Self::DecPrecision(sub, ..) => vec![sub.clone()],
            Self::SetPrecision(sub, ..) => vec![sub.clone()],
        }
    }
}

impl<R: Reference> fmt::Display for Operation<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use types::GroundType as GT;

        match self {
            Self::Add(lhs, rhs)                     => write!(f, "add({}, {})", lhs, rhs),
            Self::Sub(lhs, rhs)                     => write!(f, "sub({}, {})", lhs, rhs),
            Self::Mul(lhs, rhs)                     => write!(f, "mul({}, {})", lhs, rhs),
            Self::Div(lhs, rhs)                     => write!(f, "div({}, {})", lhs, rhs),
            Self::Rem(lhs, rhs)                     => write!(f, "rem({}, {})", lhs, rhs),
            Self::Lt(lhs, rhs)                      => write!(f, "lt({}, {})", lhs, rhs),
            Self::LEq(lhs, rhs)                     => write!(f, "leq({}, {})", lhs, rhs),
            Self::Gt(lhs, rhs)                      => write!(f, "gt({}, {})", lhs, rhs),
            Self::GEq(lhs, rhs)                     => write!(f, "geq({}, {})", lhs, rhs),
            Self::Eq(lhs, rhs)                      => write!(f, "eq({}, {})", lhs, rhs),
            Self::NEq(lhs, rhs)                     => write!(f, "neq({}, {})", lhs, rhs),
            Self::Pad(sub, bits)                    => write!(f, "pad({}, {})", sub, bits),
            Self::Cast(sub, GT::UInt(..))           => write!(f, "asUInt({})", sub),
            Self::Cast(sub, GT::SInt(..))           => write!(f, "asSInt({})", sub),
            Self::Cast(sub, GT::Fixed(.., Some(p))) => write!(f, "asFixed({}, {})", sub, p),
            Self::Cast(sub, GT::Clock)              => write!(f, "asClock({})", sub),
            Self::Cast(..)                          => Err(Default::default()),
            Self::Shl(sub, bits)                    => write!(f, "shl({}, {})", sub, bits),
            Self::Shr(sub, bits)                    => write!(f, "shr({}, {})", sub, bits),
            Self::DShl(sub, bits)                   => write!(f, "dshl({}, {})", sub, bits),
            Self::DShr(sub, bits)                   => write!(f, "dshr({}, {})", sub, bits),
            Self::Cvt(sub)                          => write!(f, "cvt({})", sub),
            Self::Neg(sub)                          => write!(f, "neg({})", sub),
            Self::Not(sub)                          => write!(f, "not({})", sub),
            Self::And(lhs, rhs)                     => write!(f, "and({}, {})", lhs, rhs),
            Self::Or(lhs, rhs)                      => write!(f, "or({}, {})", lhs, rhs),
            Self::Xor(lhs, rhs)                     => write!(f, "xor({}, {})", lhs, rhs),
            Self::AndReduce(sub)                    => write!(f, "andr({})", sub),
            Self::OrReduce(sub)                     => write!(f, "orr({})", sub),
            Self::XorReduce(sub)                    => write!(f, "xorr({})", sub),
            Self::Cat(lhs, rhs)                     => write!(f, "cat({}, {})", lhs, rhs),
            Self::Bits(sub, Some(l), Some(h))       => write!(f, "bits({}, {}, {})", sub, l, h),
            Self::Bits(sub, None, Some(high))       => write!(f, "head({}, {})", sub, high),
            Self::Bits(sub, Some(low), None)        => write!(f, "tail({}, {})", sub, low),
            Self::Bits(..)                          => Err(Default::default()),
            Self::IncPrecision(sub, bits)           => write!(f, "incp({}, {})", sub, bits),
            Self::DecPrecision(sub, bits)           => write!(f, "decp({}, {})", sub, bits),
            Self::SetPrecision(sub, bits)           => write!(f, "setp({}, {})", sub, bits),
        }
    }
}

#[cfg(test)]
impl Arbitrary for Operation<crate::tests::Identifier> {
    fn arbitrary(g: &mut Gen) -> Self {
        use types::GroundType as GT;

        // We want to exclude the analog types
        let opts = [
            GT::UInt(Arbitrary::arbitrary(g)),
            GT::SInt(Arbitrary::arbitrary(g)),
            GT::Fixed(Arbitrary::arbitrary(g), Arbitrary::arbitrary(g)),
            GT::Clock,
        ];

        super::tests::primitive_op_with_type(*Gen::new(g.size() / 10).choose(&opts).unwrap(), g)
    }

    /// Shrink the expressions within this primitive op
    ///
    /// Note that for primitive operations, our shrinking goal deviates from our
    /// usual goal for recursive structures.
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        use crate::tests::Identifier;

        let bin_shrink = |l: &Arc<Expression<Identifier>>, r: &Arc<Expression<Identifier>>| {
            let r = r.clone();
            l.shrink()
                .flat_map(move |l| std::iter::once(r.clone()).chain(r.shrink()).map(move |r| (l.clone(), r)))
        };

        match self {
            Self::Add(lhs, rhs)         => Box::new(bin_shrink(lhs, rhs).map(|(l, r)| Self::Add(l, r))),
            Self::Sub(lhs, rhs)         => Box::new(bin_shrink(lhs, rhs).map(|(l, r)| Self::Sub(l, r))),
            Self::Mul(lhs, rhs)         => Box::new(bin_shrink(lhs, rhs).map(|(l, r)| Self::Mul(l, r))),
            Self::Div(lhs, rhs)         => Box::new(bin_shrink(lhs, rhs).map(|(l, r)| Self::Div(l, r))),
            Self::Rem(lhs, rhs)         => Box::new(bin_shrink(lhs, rhs).map(|(l, r)| Self::Rem(l, r))),
            Self::Lt(lhs, rhs)          => Box::new(bin_shrink(lhs, rhs).map(|(l, r)| Self::Lt(l, r))),
            Self::LEq(lhs, rhs)         => Box::new(bin_shrink(lhs, rhs).map(|(l, r)| Self::LEq(l, r))),
            Self::Gt(lhs, rhs)          => Box::new(bin_shrink(lhs, rhs).map(|(l, r)| Self::Gt(l, r))),
            Self::GEq(lhs, rhs)         => Box::new(bin_shrink(lhs, rhs).map(|(l, r)| Self::GEq(l, r))),
            Self::Eq(lhs, rhs)          => Box::new(bin_shrink(lhs, rhs).map(|(l, r)| Self::Eq(l, r))),
            Self::NEq(lhs, rhs)         => Box::new(bin_shrink(lhs, rhs).map(|(l, r)| Self::NEq(l, r))),
            Self::Pad(sub, bits)        => {
                let b = *bits;
                Box::new(sub.shrink().map(move |e| Self::Pad(e, b)))
            },
            Self::Cast(sub, t)          => {
                let t = *t;
                Box::new(sub.shrink().map(move |e| Self::Cast(e, t)))
            },
            Self::Shl(sub, bits)        => {
                let b = *bits;
                Box::new(sub.shrink().map(move |e| Self::Shl(e, b)))
            },
            Self::Shr(sub, bits)        => {
                let b = *bits;
                Box::new(sub.shrink().map(move |e| Self::Shr(e, b)))
            },
            Self::DShl(sub, i)          => Box::new(bin_shrink(sub, i).map(|(l, r)| Self::DShl(l, r))),
            Self::DShr(sub, i)          => Box::new(bin_shrink(sub, i).map(|(l, r)| Self::DShr(l, r))),
            Self::Cvt(sub)              => Box::new(sub.shrink().map(Self::Cvt)),
            Self::Neg(sub)              => Box::new(sub.shrink().map(Self::Neg)),
            Self::Not(sub)              => Box::new(sub.shrink().map(Self::Not)),
            Self::And(lhs, rhs)         => Box::new(bin_shrink(lhs, rhs).map(|(l, r)| Self::And(l, r))),
            Self::Or(lhs, rhs)          => Box::new(bin_shrink(lhs, rhs).map(|(l, r)| Self::Or(l, r))),
            Self::Xor(lhs, rhs)         => Box::new(bin_shrink(lhs, rhs).map(|(l, r)| Self::Xor(l, r))),
            Self::AndReduce(sub)        => Box::new(sub.shrink().map(Self::AndReduce)),
            Self::OrReduce(sub)         => Box::new(sub.shrink().map(Self::OrReduce)),
            Self::XorReduce(sub)        => Box::new(sub.shrink().map(Self::XorReduce)),
            Self::Cat(lhs, rhs)         => Box::new(bin_shrink(lhs, rhs).map(|(l, r)| Self::Cat(l, r))),
            Self::Bits(sub, h, l)       => {
                let h = *h;
                let l = *l;
                Box::new(sub.shrink().map(move |e| Self::Bits(e, h, l)))
            },
            Self::IncPrecision(sub, b)  => {
                let b = *b;
                Box::new(sub.shrink().map(move |e| Self::IncPrecision(e, b)))
            },
            Self::DecPrecision(sub, b)  => {
                let b = *b;
                Box::new(sub.shrink().map(move |e| Self::DecPrecision(e, b)))
            },
            Self::SetPrecision(sub, b)  => {
                let b = *b;
                Box::new(sub.shrink().map(move |e| Self::SetPrecision(e, b)))
            },
        }
    }
}

