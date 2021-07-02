//! Parsers for modules and related items

use std::sync::Arc;

use nom::branch::alt;
use nom::combinator::{iterator, map, value};
use nom::sequence::tuple;

use crate::indentation::Indentation;
use crate::info::{WithInfo, parse as parse_info};
use crate::parsers::{IResult, identifier, kw, le, op, spaced};
use crate::stmt::parsers::stmts;
use crate::types::parsers::r#type;


/// Utility for parsing dependant modules
#[derive(Default, Debug)]
pub struct Modules {
    modules: std::collections::HashMap<Arc<str>, Arc<super::Module>>,
}

impl Modules {
    /// Parse one module
    pub fn parse_module<'i>(
        &mut self,
        input: &'i str,
        indentation: &'_ mut Indentation,
    ) -> IResult<'i, Arc<super::Module>> {
        module(|name| self.module(name).cloned(), input, indentation).map(|(i, m)| {
            let module = Arc::new(m);
            self.modules.insert(module.name.clone(), module.clone());
            (i, module)
        })
    }

    /// Retrieve a previously parsed module by name
    pub fn module(&self, name: impl AsRef<str>) -> Option<&Arc<super::Module>> {
        self.modules.get(name.as_ref())
    }
}

impl std::iter::FromIterator<Arc<super::Module>> for Modules {
    fn from_iter<I: IntoIterator<Item = Arc<super::Module>>>(modules: I) -> Self {
        Self {modules: modules.into_iter().map(|m| (m.name.clone(), m)).collect()}
    }
}


/// Parse a Module
pub fn module<'i>(
    module: impl Fn(&str) -> Option<Arc<super::Module>> + Copy,
    input: &'i str,
    indentation: &'_ mut Indentation,
) -> IResult<'i, super::Module> {
    let (input, (name, kind, info)) = map(
        tuple((indentation.parser(), kind, spaced(identifier), spaced(op(":")), parse_info, le)),
        |(_, kind, name, _, info, ..)| (name.into(), kind, info)
    )(input)?;

    let mut indentation = indentation.sub();

    let mut ports = iterator(input, map(tuple((indentation.parser(), port, le)), |(_, p, ..)| Arc::new(p)));
    let mut res = super::Module::new(name, &mut ports, kind);
    res.set_info(info);
    let (input, _) = ports.finish()?;

    let input = match kind {
        super::Kind::Regular => {
            let (input, stmts) = stmts(
                |n| res.port_by_name(&n).map(|p| Arc::new(p.clone().into())),
                module,
                input,
                &mut indentation,
            )?;

            res.statements_mut().map(|s| *s = stmts);
            input
        },
        super::Kind::External => input,
    };

    Ok((input, res))
}


/// Parse a module kind
pub fn kind<'i>(input: &str) -> IResult<super::Kind> {
    alt((
        value(super::Kind::Regular,  kw("module")),
        value(super::Kind::External, kw("extmodule")),
    ))(input)
}


/// Parse a module instance
pub fn instance<'i>(
    module: impl Fn(&str) -> Option<Arc<super::Module>>,
    input: &'i str,
) -> IResult<'i, super::Instance> {
    nom::combinator::map_opt(
        tuple((kw("inst"), spaced(identifier), spaced(kw("of")), spaced(identifier))),
        |(_, inst_name, _, mod_name)| module(mod_name).map(|m| super::Instance::new(inst_name, m)),
    )(input)
}


/// Parse the elements of a port
pub fn port<'i>(input: &str) -> IResult<super::Port> {
    map(
        tuple((direction, spaced(identifier), spaced(op(":")), spaced(r#type), parse_info)),
        |(direction, name, _, r#type, info)| super::Port::new(name.to_string(), r#type, direction)
            .with_info(info)
    )(input)
}


/// Parse a direction
pub fn direction(input: &str) -> IResult<super::Direction> {
    use super::Direction as D;

    alt((
        value(D::Input, kw("input")),
        value(D::Output, kw("output")),
    ))(input)
}

