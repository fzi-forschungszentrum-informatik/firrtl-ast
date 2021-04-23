//! Primitive operations

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

