//! Utilities for implementation of Display

use std::fmt;

use crate::indentation::{DisplayIndented, Indentation};
use crate::info::Info;

use super::print;


/// Utility for displaying an entity declaration
pub(crate) struct EntityDecl<'a>(pub &'a super::Entity, pub Info<'a>);

impl DisplayIndented for EntityDecl<'_> {
    fn fmt<W: fmt::Write>(&self, indentation: &mut Indentation, f: &mut W) -> fmt::Result {
        use crate::memory::display::MemoryDecl;

        use super::Entity as E;

        match self.0 {
            E::Port(_)              => Err(Default::default()),
            E::Wire{name, r#type}   =>
                writeln!(f, "{}wire {}: {}{}", indentation.lock(), name, r#type, self.1),
            E::Register(reg)        => writeln!(f, "{}{}{}", indentation.lock(), reg, self.1),
            E::Node{name, value}    =>
                writeln!(f, "{}node {} = {}{}", indentation.lock(), name, value, self.1),
            E::Memory(mem)          => MemoryDecl(mem, self.1.clone()).fmt(indentation, f),
            E::SimpleMemPort(port)  => writeln!(f, "{}{}{}", indentation.lock(), port, self.1),
            E::Instance(inst)       => writeln!(f, "{}{}{}", indentation.lock(), inst, self.1),
        }
    }
}


/// Utility for rendering a format string
pub struct FormatString<'a>(pub &'a [print::PrintElement]);

impl fmt::Display for FormatString<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use print::PrintElement as P;
        use print::Format as F;

        write!(f, "\"")?;
        for element in self.0 {
            match element {
                P::Literal(s)               => s.chars().try_for_each(|c| match c {
                    '%'  => write!(f, "%%"),
                    '\n' => write!(f, "\\n"),
                    '\t' => write!(f, "\\t"),
                    '\\' => write!(f, "\\\\"),
                    '"'  => write!(f, "\\\""),
                    '\'' => write!(f, "\\'"),
                    c    => fmt::Display::fmt(&c, f),
                }),
                P::Value(_, F::Binary)      => write!(f, "%b"),
                P::Value(_, F::Decimal)     => write!(f, "%d"),
                P::Value(_, F::Hexadecimal) => write!(f, "%x"),
                P::Value(_, F::Character)   => write!(f, "%c"),
            }?
        }
        write!(f, "\"")
    }
}


/// Utility for formatting an optional name for a "special" statement
pub struct OptionalName<'a>(pub Option<&'a str>);

impl<'a> From<Option<&'a str>> for OptionalName<'a> {
    fn from(name: Option<&'a str>) -> Self {
        Self(name)
    }
}

impl fmt::Display for OptionalName<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.as_ref().map(|n| write!(f, " : {}", n)).transpose().map(|_| ())
    }
}


/// Utility for displaying a list of statements
pub struct StatementList<'a>(pub &'a [super::Statement]);

impl DisplayIndented for StatementList<'_> {
    fn fmt<W: fmt::Write>(&self, indent: &mut Indentation, f: &mut W) -> fmt::Result {
        if self.0.len() > 0 {
            self.0.iter().try_for_each(|s| s.fmt(indent, f))
        } else {
            super::Statement::from(super::Kind::Empty).fmt(indent, f)
        }
    }
}

