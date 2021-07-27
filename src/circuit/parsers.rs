// Copyright (c) 2021 FZI Forschungszentrum Informatik
// SPDX-License-Identifier: Apache-2.0
//! Parsers for Circuits

use nom::combinator::map;
use nom::multi::fold_many0;
use nom::sequence::tuple;

use crate::error::{ParseError, convert_error};
use crate::info::parse as parse_info;
use crate::module::parsers::Modules;
use crate::parsers::{identifier, kw, le, op, spaced};


/// Parse a Circuit
pub fn circuit(input: &str) -> Result<super::Circuit, ParseError> {
    consumer(input)?.into_circuit()
}


/// Create a ModuleConsumer for the given input
///
/// The input is expected to contain a full circuit definition. The function
/// will return a `ModuleConsumer` which will construct a `Circuit` from that
/// input.
pub fn consumer(input: &str) -> Result<super::ModuleConsumer<Modules, ParseError>, ParseError> {
    let (mod_input, (top_name, info)) = map(
        tuple((
            fold_many0(le, (), |_, _| ()),
            kw("circuit"),
            spaced(identifier),
            spaced(op(":")),
            parse_info,
            le,
        )),
        |(_, _, n, _, i, ..)| (n, i)
    )(input).map_err(|e| convert_error(input, e))?;

    Ok(super::ModuleConsumer::new(top_name, info, Modules::new_with_origin(mod_input, input)))
}

