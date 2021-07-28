// Copyright (c) 2021 FZI Forschungszentrum Informatik
// SPDX-License-Identifier: Apache-2.0
//! Tests related to circuits

use quickcheck::{Gen, TestResult, Testable};

use crate::error::ParseError;
use crate::named::Named;
use crate::tests::Equivalence;

use super::{Circuit, parsers};


#[quickcheck]
fn parse_circuit(original: Circuit) -> Result<TestResult, ParseError> {
    use transiter::IntoTransIter;

    // Module names must be unique within a circuit. If they are not, the set of
    // names will be smaller than the number of instantiations generated from.
    let mut mod_num = 0;
    let mods = original
        .top_module()
        .trans_iter_with(|m| m.referenced_modules())
        .inspect(|_| mod_num = mod_num + 1)
        .map(|i| i.name())
        .collect::<std::collections::HashSet<_>>();
    if mods.len() != mod_num {
        return Ok(TestResult::discard())
    }

    let s = original.to_string();
    parsers::circuit(&s)
        .map(|parsed| Equivalence::of(original, parsed).result(&mut Gen::new(0)))
}
