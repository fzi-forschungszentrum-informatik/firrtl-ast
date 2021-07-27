// Copyright (c) 2021 FZI Forschungszentrum Informatik
// SPDX-License-Identifier: Apache-2.0
//! Display utilties for memories

use std::fmt;

use crate::indentation::{DisplayIndented, Indentation};
use crate::info::Info;


pub(crate) struct MemoryDecl<'a>(pub &'a super::Memory, pub Info<'a>);

impl DisplayIndented for MemoryDecl<'_> {
    fn fmt<W: fmt::Write>(&self, indentation: &mut Indentation, f: &mut W) -> fmt::Result {
        use crate::expr::Reference;

        writeln!(f, "{}mem {}:{}", indentation.lock(), self.0.name(), self.1)?;
        let mut indentation = indentation.sub();
        writeln!(f, "{}data-type => {}", indentation.lock(), self.0.data_type())?;
        writeln!(f, "{}depth => {}", indentation.lock(), self.0.depth())?;
        self.0.ports().try_for_each(|p| DisplayIndented::fmt(p, &mut indentation, f))?;
        writeln!(f, "{}read-latency => {}", indentation.lock(), self.0.read_latency())?;
        writeln!(f, "{}write-latency => {}", indentation.lock(), self.0.write_latency())?;
        writeln!(f, "{}read-under-write => {}", indentation.lock(), self.0.read_under_write())
    }
}

