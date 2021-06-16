//! Utilities for implementation of Display

use std::fmt;

use crate::indentation::{DisplayIndented, Indentation};


/// Utility for displaying an entity declaration
pub struct Entity<'a>(pub &'a super::Entity);

impl DisplayIndented for Entity<'_> {
    fn fmt<W: fmt::Write>(&self, indentation: &mut Indentation, f: &mut W) -> fmt::Result {
        use super::Entity as E;
        match self.0 {
            E::Port(_)              => Err(Default::default()),
            E::Wire{name, r#type}   => writeln!(f, "{}wire {}: {}", indentation.lock(), name, r#type),
            E::Register(reg)        => DisplayIndented::fmt(reg, indentation, f),
            E::Node{name, value}    => writeln!(f, "{}node {} = {}", indentation.lock(), name, value),
            E::Memory(mem)          => DisplayIndented::fmt(mem, indentation, f),
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
                    '%' => write!(f, "%%"),
                    c   => fmt::Display::fmt(&c.escape_default(), f),
                }),
                P::Value(_, F::Binary)      => write!(f, "%b"),
                P::Value(_, F::Decimal)     => write!(f, "%d"),
                P::Value(_, F::Hexadecimal) => write!(f, "%x"),
            }?
        }
        write!(f, "\"")
    }
}

