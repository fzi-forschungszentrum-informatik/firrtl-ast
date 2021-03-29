//! Testing utilities

use std::fmt;

/// Utility type for property-based tests involving an equivalence
///
/// Sometimes, properties we want to test for are the equivalence of two values.
/// For example, we may construct a pseudeo-identity from a formatter and a
/// parser in order to test a parser. In such cases, we want to compare the
/// input of the pseudo-identity to its output.
///
/// `Equivalence` is a `quickcheck::Testable` type which expresses this intent,
/// but also includes both values as part of the failure report if a test fails.
///
#[derive(Clone, Debug)]
pub struct Equivalence<T>(pub T, pub T)
where
    T: fmt::Debug + PartialEq + 'static;

impl<T> Equivalence<T>
where
    T: fmt::Debug + PartialEq + 'static,
{
    /// Construct a value expressing the equivalence of the given values
    ///
    /// In many cases, you'll be able to construct an instance for two values
    /// `a` and `b` via `Equivalence(a, b)`. This function is intended for
    /// situations where you can't for whatever reasons.
    pub fn of(left: T, right: T) -> Self {
        Self(left, right)
    }
}

impl<T> quickcheck::Testable for Equivalence<T>
where
    T: fmt::Debug + PartialEq + 'static,
{
    fn result(&self, _: &mut quickcheck::Gen) -> quickcheck::TestResult {
        use quickcheck::TestResult;
        if self.0 == self.1 {
            TestResult::passed()
        } else {
            TestResult::error(format!(
                "Missmatch! Left: '{:?}', Right: '{:?}'",
                self.0, self.1
            ))
        }
    }
}


