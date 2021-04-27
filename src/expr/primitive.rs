//! Primitive operations

use std::fmt;
use std::num::NonZeroU16;
use std::sync::Arc;

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

impl<R: Reference> fmt::Display for Operation<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use types::GroundType as GT;

        match self {
            Self::Add(lhs, rhs)                 => write!(f, "add({}, {})", lhs, rhs),
            Self::Sub(lhs, rhs)                 => write!(f, "sub({}, {})", lhs, rhs),
            Self::Mul(lhs, rhs)                 => write!(f, "mul({}, {})", lhs, rhs),
            Self::Div(lhs, rhs)                 => write!(f, "div({}, {})", lhs, rhs),
            Self::Rem(lhs, rhs)                 => write!(f, "rem({}, {})", lhs, rhs),
            Self::Lt(lhs, rhs)                  => write!(f, "lt({}, {})", lhs, rhs),
            Self::LEq(lhs, rhs)                 => write!(f, "leq({}, {})", lhs, rhs),
            Self::Gt(lhs, rhs)                  => write!(f, "gt({}, {})", lhs, rhs),
            Self::GEq(lhs, rhs)                 => write!(f, "geq({}, {})", lhs, rhs),
            Self::Eq(lhs, rhs)                  => write!(f, "eq({}, {})", lhs, rhs),
            Self::NEq(lhs, rhs)                 => write!(f, "neq({}, {})", lhs, rhs),
            Self::Pad(sub, bits)                => write!(f, "pad({}, {})", sub, bits),
            Self::Cast(sub, GT::UInt(..))       => write!(f, "asUInt({})", sub),
            Self::Cast(sub, GT::SInt(..))       => write!(f, "asSInt({})", sub),
            Self::Cast(sub, GT::Fixed(..))      => write!(f, "asFixed({})", sub),
            Self::Cast(sub, GT::Clock)          => write!(f, "asClock({})", sub),
            Self::Cast(..)                      => Err(Default::default()),
            Self::Shl(sub, bits)                => write!(f, "shl({}, {})", sub, bits),
            Self::Shr(sub, bits)                => write!(f, "shr({}, {})", sub, bits),
            Self::DShl(sub, bits)               => write!(f, "dshl({}, {})", sub, bits),
            Self::DShr(sub, bits)               => write!(f, "dshr({}, {})", sub, bits),
            Self::Cvt(sub)                      => write!(f, "cvt({})", sub),
            Self::Neg(sub)                      => write!(f, "neg({})", sub),
            Self::Not(sub)                      => write!(f, "not({})", sub),
            Self::And(lhs, rhs)                 => write!(f, "and({}, {})", lhs, rhs),
            Self::Or(lhs, rhs)                  => write!(f, "or({}, {})", lhs, rhs),
            Self::Xor(lhs, rhs)                 => write!(f, "xor({}, {})", lhs, rhs),
            Self::AndReduce(sub)                => write!(f, "andr({})", sub),
            Self::OrReduce(sub)                 => write!(f, "orr({})", sub),
            Self::XorReduce(sub)                => write!(f, "xorr({})", sub),
            Self::Cat(lhs, rhs)                 => write!(f, "cat({}, {})", lhs, rhs),
            Self::Bits(sub, Some(l), Some(h))   => write!(f, "bits({}, {}, {})", sub, l, h),
            Self::Bits(sub, None, Some(high))   => write!(f, "head({}, {})", sub, high),
            Self::Bits(sub, Some(low), None)    => write!(f, "tail({}, {})", sub, low),
            Self::Bits(..)                      => Err(Default::default()),
            Self::IncPrecision(sub, bits)       => write!(f, "incp({}, {})", sub, bits),
            Self::DecPrecision(sub, bits)       => write!(f, "decp({}, {})", sub, bits),
            Self::SetPrecision(sub, bits)       => write!(f, "setp({}, {})", sub, bits),
        }
    }
}

