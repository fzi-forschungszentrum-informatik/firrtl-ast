//! Indentation utilities

use std::fmt;
use std::num::NonZeroUsize;

#[cfg(test)]
use quickcheck::{Arbitrary, Gen};

use crate::parsers;

/// Print with indentation
pub trait DisplayIndented {
    /// Print the instance with the given indentation
    fn fmt<W: fmt::Write>(&self, indentation: &mut Indentation, f: &mut W) -> fmt::Result;
}

impl<T> DisplayIndented for T
    where T: fmt::Display
{
    fn fmt<W: fmt::Write>(&self, indentation: &mut Indentation, f: &mut W) -> fmt::Result {
        writeln!(f, "{}{}", indentation.lock(), self)
    }
}


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
    /// Create a new `Indentation` with a lower (i.e. nested) level
    ///
    /// This function locks the `Indentation`.
    pub fn sub(&mut self) -> Self {
        self.lock().sub()
    }

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

    /// Create a parser for this Indentation
    pub fn parser(&mut self) -> IndentationParser {
        IndentationParser{inner: self}
    }
}

impl Default for Indentation {
    fn default() -> Self {
        Self::root()
    }
}

#[cfg(test)]
impl Arbitrary for Indentation {
    fn arbitrary(g: &mut Gen) -> Self {
        // Testing huge widths will (probably) not yield any benefits.
        let i = u8::arbitrary(g) as usize;
        g.choose(&[Self::MoreThan(i), Self::Exact(i)]).unwrap().clone()
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        match self {
            Self::MoreThan(i) => Box::new(i.shrink().map(Self::MoreThan)),
            Self::Exact(i)    => Box::new(i.shrink().map(Self::Exact)),
        }
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

impl fmt::Display for LockedIndentation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use std::fmt::Write;

        (0..self.0).try_for_each(|_| f.write_char(' '))
    }
}


/// Indentation parser
///
/// This parser consumes sequences of space characters. The sequence is only
/// accepted if the length-requirement represented by the associated
/// `Indentation` is met.
pub struct IndentationParser<'a> {
    inner: &'a mut Indentation
}

impl<'i> nom::Parser<&'i str, (), parsers::Error<'i>> for IndentationParser<'_> {
    fn parse(&mut self, input: &'i str) -> parsers::IResult<'i, ()> {
        use nom::error::ParseError;

        let (rest, len) = nom::multi::many0_count(nom::character::complete::char(' '))(input)?;
        match self.inner {
            Indentation::MoreThan(l) if len > *l => *self.inner = Indentation::Exact(len),
            Indentation::Exact(l) if len == *l => (),
           _ => return Err(nom::Err::Error(parsers::Error::from_error_kind(input, nom::error::ErrorKind::Many1Count))),
        };
        Ok((rest, ()))
    }
}


/// Default number of spaces for one indentation step
const INDENTATION_STEP: usize = 2;

