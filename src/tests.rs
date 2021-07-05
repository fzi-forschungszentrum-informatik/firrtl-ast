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


/// Utility type for generating identifiers for tests
#[derive(Clone, Debug, PartialEq)]
pub struct Identifier {
    data: String
}

impl From<&str> for Identifier {
    fn from(ident: &str) -> Self {
        Self {data: ident.to_string()}
    }
}

impl From<Identifier> for std::sync::Arc<str> {
    fn from(ident: Identifier) -> Self {
        ident.data.into()
    }
}

impl AsRef<str> for Identifier {
    fn as_ref(&self) -> &str {
        self.data.as_ref()
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.data, f)
    }
}

impl quickcheck::Arbitrary for Identifier {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        // Choose some some common prefix to avoid conflicts with keywords.
        let mut data = "T".to_string();

        let mut i: u128 = quickcheck::Arbitrary::arbitrary(g);
        while i > 0 {
            const N: u128 = 2*36 + 10 + 2;
            data.push(match (i % N) as u8 {
                i if i < 10 => (0x30 + i) as char,
                i if i < 36 => (0x41 - 10 + i) as char,
                i if i < 62 => (0x61 - 36 + i) as char,
                62 => '_',
                _ => '$',
            });
            i = i / N;
        }

        Self {data}
    }
}


/// Utility type for generating non-empty ASCII strings
#[derive(Clone, Debug, PartialEq)]
pub struct ASCII {
    data: String
}

impl ASCII {
    /// Check whether the given char is valid for an ASCII string
    pub fn is_valid_char(c: char) -> bool {
        c.is_ascii() && (!c.is_ascii_control() || c == '\n' || c == '\t')
    }
}

impl From<String> for ASCII {
    fn from(data: String) -> Self {
        Self {data}
    }
}

impl AsRef<str> for ASCII {
    fn as_ref(&self) -> &str {
        self.data.as_ref()
    }
}

impl fmt::Display for ASCII {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.data, f)
    }
}

impl quickcheck::Arbitrary for ASCII {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        let len = u8::arbitrary(g).saturating_add(1) as usize;
        std::iter::from_fn(|| Some(char::arbitrary(g)))
            .filter(|c| Self::is_valid_char(*c))
            .take(len)
            .collect::<String>()
            .into()
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        let res = self
            .data
            .shrink()
            .filter(|s| !s.is_empty() && s.chars().all(Self::is_valid_char))
            .map(Into::into);
        Box::new(res)
    }
}

