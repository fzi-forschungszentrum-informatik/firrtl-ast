//! Utilities for implementation of Display

use std::fmt;

use crate::indentation::{DisplayIndented, Indentation};


/// Utility for displaying an entity declaration
pub struct EntityDecl<'a>(pub &'a super::Entity);

impl DisplayIndented for EntityDecl<'_> {
    fn fmt<W: fmt::Write>(&self, indentation: &mut Indentation, f: &mut W) -> fmt::Result {
        use crate::memory::display::MemoryDecl;

        use super::Entity as E;

        match self.0 {
            E::Port(_)              => Err(Default::default()),
            E::Wire{name, r#type}   => writeln!(f, "{}wire {}: {}", indentation.lock(), name, r#type),
            E::Register(reg)        => DisplayIndented::fmt(reg, indentation, f),
            E::Node{name, value}    => writeln!(f, "{}node {} = {}", indentation.lock(), name, value),
            E::Memory(mem)          => MemoryDecl(mem, Default::default()).fmt(indentation, f),
            E::Instance(inst)       => DisplayIndented::fmt(inst, indentation, f),
        }
    }
}


/// Utility for rendering a format string
pub struct FormatString<'a>(pub &'a [super::PrintElement]);

impl fmt::Display for FormatString<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use super::PrintElement as P;
        use super::Format as F;

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
            }?
        }
        write!(f, "\"")
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

