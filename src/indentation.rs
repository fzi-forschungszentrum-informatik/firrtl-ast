//! Indentation utilities

use std::num::NonZeroUsize;


/// Indentation
///
/// Instances of this type represent either a lower bound or an exact length of
/// a sequences of space characters. Usually, a new `Indentation` will represent
/// a lower bound. After an instance has been locked, it will always refer to
/// the same exact length.
#[derive(Clone, Debug, PartialEq)]
pub enum Indentation {
    MoreThan(usize),
    Exact(usize),
}

impl Indentation {
    /// Lock the indentation to a concrete value
    ///
    /// If the `Indentation` represents only a lower bound, this function will
    /// set an exact value which will be some fixed value above the parent
    /// indentation level. If the `Indentation` is already locked, this function
    /// doesn't have any effect.
    ///
    /// The function returns a `LockedIndentation` reflecting the excact
    /// indentation length.
    pub fn lock(&mut self) -> LockedIndentation {
        self.lock_with(NonZeroUsize::new(INDENTATION_STEP).expect("Invalid indentation width"))
    }

    /// Lock the indentation to a concrete value
    ///
    /// If the `Indentation` represents only a lower bound, this function will
    /// set an exact value which will be `steps` above the parent indentation
    /// level. If the `Indentation` is already locked, this function doesn't
    /// have any effect.
    ///
    /// The function returns a `LockedIndentation` reflecting the excact
    /// indentation length.
    pub fn lock_with(&mut self, steps: NonZeroUsize) -> LockedIndentation {
        match self {
            Self::MoreThan(i) => {
                let i = *i + steps.get();
                *self = Self::Exact(i);
                LockedIndentation(i)
            },
            Self::Exact(i) => LockedIndentation(*i),
        }
    }

    /// Create a new, locked "root"
    ///
    /// The `Indentation` returned will be locked to a length of `0`, i.e. no
    /// indentation at all.
    pub fn root() -> Self {
        Self::Exact(0)
    }
}

impl Default for Indentation {
    fn default() -> Self {
        Self::root()
    }
}


#[derive(Copy, Clone, Debug)]
pub struct LockedIndentation(usize);

impl LockedIndentation {
    /// Create a new indentation with a lower (i.e. nested) level
    pub fn sub(&self) -> Indentation {
        Indentation::MoreThan(self.into())
    }
}

impl From<&LockedIndentation> for usize {
    fn from(i: &LockedIndentation) -> Self {
        i.0
    }
}

impl From<LockedIndentation> for usize {
    fn from(i: LockedIndentation) -> Self {
        i.0
    }
}


/// Default number of spaces for one indentation step
const INDENTATION_STEP: usize = 2;

