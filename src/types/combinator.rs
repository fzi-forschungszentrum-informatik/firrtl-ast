//! Combinator trait and implementations

use super::BitWidth;


/// Combinator for two entities
///
/// This trait provides an interface for combining two entities. Implementations
/// define how the combination is carried out.
pub trait Combinator<T> {
    /// Combine two entities
    ///
    /// This function combines two entities. If the combination fails, it
    /// returns the pair responsible for the failure. In many cases, those will
    /// be the input parameters.
    fn combine<'a>(&self, lhs: &'a T, rhs: &'a T) -> Result<T, (&'a T, &'a T)>;
}


/// Combinator for two known widths
///
/// This `Combinator` will return a width from two known widths computed via a
/// given function. If one of the width is unknown, the `Combinator` yields an
/// unknown width (i.e. `None`). It never yields an error.
pub struct FnWidth<F: Fn(u16, u16) -> Option<u16>> {
    inner: F
}

impl<F: Fn(u16, u16) -> Option<u16>> FnWidth<F> {
    /// Combine two widths
    ///
    /// This function performs the combination, but returns the result bare,
    /// i.e. not as a `Result`.
    pub fn combine_widths(&self, lhs: BitWidth, rhs: BitWidth) -> BitWidth {
        if let (Some(l), Some(r)) = (lhs, rhs) {
            (self.inner)(l, r)
        } else {
            None
        }
    }
}

impl<F: Fn(u16, u16) -> Option<u16>> Combinator<BitWidth> for FnWidth<F> {
    fn combine<'a>(
        &self,
        lhs: &'a BitWidth,
        rhs: &'a BitWidth
    ) -> Result<BitWidth, (&'a BitWidth, &'a BitWidth)> {
        Ok(self.combine_widths(*lhs, *rhs))
    }
}

impl<F: Fn(u16, u16) -> Option<u16>> From<F> for FnWidth<F> {
    fn from(inner: F) -> Self {
        Self {inner}
    }
}

