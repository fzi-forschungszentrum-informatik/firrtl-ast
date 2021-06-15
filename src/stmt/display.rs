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

