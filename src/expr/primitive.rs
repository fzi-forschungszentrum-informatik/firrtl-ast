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

impl<R: Reference> Operation<R> {
    /// Retrieve all subexpressions used in the operation
    ///
    pub fn sub_exprs(&self) -> Vec<&Arc<Expression<R>>> {
        match self {
            Self::Add(lhs, rhs)         => vec![lhs, rhs],
            Self::Sub(lhs, rhs)         => vec![lhs, rhs],
            Self::Mul(lhs, rhs)         => vec![lhs, rhs],
            Self::Div(lhs, rhs)         => vec![lhs, rhs],
            Self::Rem(lhs, rhs)         => vec![lhs, rhs],
            Self::Lt(lhs, rhs)          => vec![lhs, rhs],
            Self::LEq(lhs, rhs)         => vec![lhs, rhs],
            Self::Gt(lhs, rhs)          => vec![lhs, rhs],
            Self::GEq(lhs, rhs)         => vec![lhs, rhs],
            Self::Eq(lhs, rhs)          => vec![lhs, rhs],
            Self::NEq(lhs, rhs)         => vec![lhs, rhs],
            Self::Pad(sub, ..)          => vec![sub],
            Self::Cast(sub, ..)         => vec![sub],
            Self::Shl(sub, ..)          => vec![sub],
            Self::Shr(sub, ..)          => vec![sub],
            Self::DShl(sub, index)      => vec![sub, index],
            Self::DShr(sub, index)      => vec![sub, index],
            Self::Cvt(sub)              => vec![sub],
            Self::Neg(sub)              => vec![sub],
            Self::Not(sub)              => vec![sub],
            Self::And(lhs, rhs)         => vec![lhs, rhs],
            Self::Or(lhs, rhs)          => vec![lhs, rhs],
            Self::Xor(lhs, rhs)         => vec![lhs, rhs],
            Self::AndReduce(sub)        => vec![sub],
            Self::OrReduce(sub)         => vec![sub],
            Self::XorReduce(sub)        => vec![sub],
            Self::Cat(lhs, rhs)         => vec![lhs, rhs],
            Self::Bits(sub, ..)         => vec![sub],
            Self::IncPrecision(sub, ..) => vec![sub],
            Self::DecPrecision(sub, ..) => vec![sub],
            Self::SetPrecision(sub, ..) => vec![sub],
        }
    }
}

impl<R> types::Typed for Operation<R>
    where R: Reference + types::Typed + Clone,
          R::Type: Into<types::Type>,
{
    type Err = Expression<R>;

    type Type = types::GroundType;

    fn r#type(&self) -> Result<Self::Type, Self::Err> {
        use std::cmp::{max, min};
        use std::convert::TryInto;

        use types::{BitWidth, Combinator, GroundType as GT, TypeExt, combinator};
        use combinator::FnWidth;

        let max_width = |l: BitWidth, r: BitWidth| types::MaxWidth::combine_widths(l, r);
        let sum_width = |l: BitWidth, r: BitWidth| FnWidth::from(u16::checked_add)
            .combine_widths(l, r);

        let ground = |e: &Arc<Expression<R>>| e
            .r#type()
            .and_then(|t| t.ground_type().ok_or_else(|| self.clone().into()));
        let fixed = |e: &Arc<Expression<R>>| ground(e).and_then(|t| if let GT::Fixed(w, p) = t {
            Ok((w, p))
        } else {
            Err(self.clone().into())
        });

        // Common logic for "sums", i.e. add and sub
        let sum = |l: &Arc<Expression<R>>, r: &Arc<Expression<R>>| match (ground(l)?, ground(r)?) {
            (GT::UInt(l), GT::UInt(r)) => Ok(GT::UInt(max_width(l, r).and_then(|w| w.checked_add(1)))),
            (GT::SInt(l), GT::SInt(r)) => Ok(GT::SInt(max_width(l, r).and_then(|w| w.checked_add(1)))),
            (GT::Fixed(Some(lw), Some(lp)), GT::Fixed(Some(rw), Some(rp))) => Ok(GT::Fixed(
                types::combine_fixed_max((lw, lp), (rw, rp)).and_then(|w| w.checked_add(1)),
                Some(max(lp, rp))
            )),
            (GT::Fixed(..), GT::Fixed(..)) => Ok(GT::Fixed(None, None)),
            _ => Err(self.clone().into()),
        };

        // Common logic for bit-wise binary ops
        let bitbin = |l: &Arc<Expression<R>>, r: &Arc<Expression<R>>| Ok(
            GT::UInt(max_width(ground(l)?.width(), ground(r)?.width()))
        );

        match self {
            Self::Add(lhs, rhs)             => sum(lhs, rhs),
            Self::Sub(lhs, rhs)             => sum(lhs, rhs),
            Self::Mul(lhs, rhs)             => match (ground(lhs)?, ground(rhs)?) {
                (GT::UInt(l), GT::UInt(r)) => Ok(GT::UInt(sum_width(l, r))),
                (GT::SInt(l), GT::SInt(r)) => Ok(GT::SInt(sum_width(l, r))),
                (GT::Fixed(lw, lp), GT::Fixed(rw, rp)) => Ok(GT::Fixed(sum_width(lw, rw), match (lp, rp) {
                    (Some(l), Some(r)) => l.checked_add(r),
                    _ => None
                })),
                _ => Err(self.clone().into()),
            },
            Self::Div(lhs, ..)              => ground(lhs).and_then(|t| match t {
                GT::UInt(w) => Ok(GT::UInt(w)),
                GT::SInt(w) => Ok(GT::SInt(w.and_then(|w| w.checked_add(1)))),
                _ => Err(self.clone().into()),
            }),
            Self::Rem(lhs, rhs)             => FnWidth::from(|l, r| Some(min(l, r)))
                .combine(&ground(lhs)?, &ground(rhs)?)
                .map_err(|_| self.clone().into()),
            Self::Lt(..)                    => Ok(GT::UInt(Some(1))),
            Self::LEq(..)                   => Ok(GT::UInt(Some(1))),
            Self::Gt(..)                    => Ok(GT::UInt(Some(1))),
            Self::GEq(..)                   => Ok(GT::UInt(Some(1))),
            Self::Eq(..)                    => Ok(GT::UInt(Some(1))),
            Self::NEq(..)                   => Ok(GT::UInt(Some(1))),
            Self::Pad(sub, bits)            => ground(sub)
                .map(|t| t.with_width(max(t.width(), Some(bits.get())))),
            Self::Cast(sub, target)         => ground(sub).map(|t| target.with_width(t.width())),
            Self::Shl(sub, bits)            => ground(sub)
                .map(|t| t.with_width(t.width().and_then(|w| w.checked_add(bits.get())))),
            Self::Shr(sub, bits)            => ground(sub).and_then(|t| match t {
                GT::UInt(w)                 => Ok(GT::UInt(w.map(|w| max(w.saturating_sub(bits.get()), 1)))),
                GT::SInt(w)                 => Ok(GT::SInt(w.map(|w| max(w.saturating_sub(bits.get()), 1)))),
                GT::Fixed(Some(w), Some(p)) => Ok(
                    GT::Fixed(w.checked_sub(bits.get()).map(|w| max(w, max(p, 1) as u16)), Some(p))
                ),
                GT::Fixed(..)               => Ok(GT::Fixed(None, None)),
                _ => Err(self.clone().into()),
            }),
            Self::DShl(sub, bits)           => ground(sub).and_then(|t| Ok(t
                .with_width(match (t.width(), ground(bits)?.width()) {
                    (Some(ws), Some(wb)) => 1u16
                        .checked_shl(wb.into())
                        .and_then(|w| w.checked_add(ws))
                        .map(|w| w - 1),
                    _ => None,
                })
            )),
            Self::DShr(sub, _)              => ground(sub),
            Self::Cvt(sub)                  => ground(sub).and_then(|t| match t {
                GT::UInt(w) => Ok(GT::SInt(w.and_then(|w| w.checked_add(1)))),
                GT::SInt(w) => Ok(GT::SInt(w)),
                _ => Err(self.clone().into()),
            }),
            Self::Neg(sub)                  => ground(sub)
                .map(|t| GT::SInt(t.width().and_then(|w| w.checked_add(1)))),
            Self::Not(sub)                  => ground(sub).map(|t| GT::UInt(t.width())),
            Self::And(lhs, rhs)             => bitbin(lhs, rhs),
            Self::Or(lhs, rhs)              => bitbin(lhs, rhs),
            Self::Xor(lhs, rhs)             => bitbin(lhs, rhs),
            Self::AndReduce(..)             => Ok(GT::UInt(Some(1))),
            Self::OrReduce(..)              => Ok(GT::UInt(Some(1))),
            Self::XorReduce(..)             => Ok(GT::UInt(Some(1))),
            Self::Cat(lhs, rhs)             => Ok(
                GT::UInt(max_width(ground(lhs)?.width(), ground(rhs)?.width())
            )),
            Self::Bits(sub, low, high)      => ground(sub).map(|t| GT::UInt(high
                .map(NonZeroU16::get)
                .or(t.width())
                .and_then(|w| w.checked_sub(low.map(NonZeroU16::get).unwrap_or(1)))
                .map(|w| w + 1)
            )),
            Self::IncPrecision(sub, bits)   => fixed(sub).map(|(w, p)| GT::Fixed(
                w.and_then(|w| w.checked_add(bits.get())),
                p.and_then(|p| p.checked_add(bits.get() as i16))
            )),
            Self::DecPrecision(sub, bits)   => fixed(sub).map(|(w, p)| GT::Fixed(
                w.and_then(|w| w.checked_sub(bits.get())),
                p.and_then(|p| p.checked_sub(bits.get() as i16))
            )),
            Self::SetPrecision(sub, bits)   => fixed(sub).map(|(w, p)| GT::Fixed(
                w.and_then(|w| p.and_then(|p| (w as i16).checked_sub(p)))
                    .and_then(|w| w.checked_add(*bits))
                    .and_then(|w| w.try_into().ok()),
                p
            )),
        }
    }
}

impl<R: Reference> fmt::Display for Operation<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use types::{GroundType as GT, ResetKind as RK};

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
            Self::Cast(sub, GT::Reset(RK::Async))   => write!(f, "asAsyncReset({})", sub),
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

