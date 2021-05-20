//! Combinator trait and implementations


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

