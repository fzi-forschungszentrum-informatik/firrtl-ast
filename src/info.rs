//! Info attribute

use std::fmt;


/// Trait providing access to attached info
///
/// FIRRTL defines an optional info attribute for some entities. The info will
/// usually hold information about where (in a generator's source) the entity
/// was generated.
pub trait WithInfo {
    /// Retrieve info attribute
    ///
    /// If no info is attached to the entity, this function will return `None`
    fn info(&self) -> Option<&str>;

    /// Set the info attribute
    fn set_info(&mut self, info: Option<String>);

    /// Set the info attribute
    fn with_info(mut self, info: Option<String>) -> Self
    where Self: Sized
    {
        self.set_info(info);
        self
    }

    /// Clear the attached info
    fn clear_info(&mut self) {
        self.set_info(None)
    }
}


/// Helper for formatting an entities info attribute
#[derive(Clone, Default)]
pub(crate) struct Info<'a>(pub Option<&'a str>);

impl<'a> Info<'a> {
    /// Create a formatting helper for the info of the given entity
    pub fn of(entity: &'a impl WithInfo) -> Self {
        entity.info().into()
    }
}

impl<'a> From<&'a str> for Info<'a> {
    fn from(i: &'a str) -> Self {
        Some(i).into()
    }
}

impl<'a> From<Option<&'a str>> for Info<'a> {
    fn from(i: Option<&'a str>) -> Self {
        Self(i)
    }
}

impl fmt::Display for Info<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(s) = self.0 {
            write!(f, " @[")?;
            s.chars().try_for_each(|c| match c {
                '\n' => write!(f, "\\\n"),
                '\t' => write!(f, "\\\t"),
                ']'  => write!(f, "\\]"),
                '\\' => write!(f, "\\\\"),
                c    => fmt::Display::fmt(&c, f),
            })?;
            write!(f, "]")
        } else {
            Ok(())
        }
    }
}

