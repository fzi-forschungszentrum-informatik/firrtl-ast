//! Genderal display utilities

use std::fmt;


/// Utility for formatting comma separated lists
pub struct CommaSeparated<'a, T: fmt::Display>(&'a [T]);

impl<'a, R, T> From<&'a R> for CommaSeparated<'a, T>
where R: AsRef<[T]>,
      T: fmt::Display,
{
    fn from(vec: &'a R) -> Self {
        Self(vec.as_ref())
    }
}

impl<'a, T: fmt::Display> fmt::Display for CommaSeparated<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut items = self.0.iter();
        items.next().map(|item| item.fmt(f)).transpose().map(|_| ())?;
        items.try_for_each(|item| write!(f, ", {}", item))
    }
}

