//! Parsers for modules and related items

use std::sync::Arc;

use nom::branch::alt;
use nom::combinator::{iterator, map, value};
use nom::sequence::tuple;

use crate::error::{ParseError, convert_error};
use crate::indentation::Indentation;
use crate::info::{WithInfo, parse as parse_info};
use crate::parsers::{IResult, identifier, kw, le, op, spaced};
use crate::stmt::parsers::stmts;
use crate::types::parsers::r#type;


/// Module iterator
///
/// This `Iterator` will yield `Module`s parsed from a given input in the order
/// they are defined in. Once a module was successfully parsed, it is availible
/// for instantiation in subsequent modules.
#[derive(Debug)]
pub struct Modules<'i> {
    modules: std::collections::HashMap<Arc<str>, Arc<super::Module>>,
    origin: &'i str,
    current: &'i str,
    indentation: Indentation,
}

impl<'i> Modules<'i> {
    /// Create a new module iterator for a given input
    ///
    /// The iterator will yield all modules from the given input in the order
    /// they are defined in.
    ///
    /// # Note
    ///
    /// The line numbers reported in case of an error will be relative to the
    /// supplied `input`. Consider using `new_with_origin` instead.
    pub fn new(input: &'i str) -> Self {
        Self::new_with_origin(input, input)
    }

    /// Create a new module iterator for a given input
    ///
    /// The iterator will yield all modules from the given input in the order
    /// they are defined in. The `original` parameter will be used for computing
    /// offsets during for error reporting.
    pub fn new_with_origin(input: &'i str, origin: &'i str) -> Self {
        Self {modules: Default::default(), origin, current: input, indentation: Indentation::root().sub()}
    }

    /// Retrieve a previously parsed module by name
    pub fn module(&self, name: impl AsRef<str>) -> Option<&Arc<super::Module>> {
        self.modules.get(name.as_ref())
    }

    /// Add a module to the list of known modules
    ///
    /// Parsed modules will be able to instantiate the added `Module`.
    pub fn add_module(&mut self, module: Arc<super::Module>) {
        self.modules.insert(module.name.clone(), module.clone());
    }
}

impl Iterator for Modules<'_> {
    type Item = Result<Arc<super::Module>, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.current.is_empty() {
            let modules = &self.modules;

            let res = module(|name| modules.get(name).cloned(), self.current, &mut self.indentation)
                .map(|(i, m)| {
                    let module = Arc::new(m);
                    self.add_module(module.clone());
                    self.current = i;
                    module
                })
                .map_err(|e| {
                    self.current = self.current.split_at(self.current.len()).1;
                    convert_error(self.origin, e)
                });
            Some(res)
        } else {
            None
        }
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
    let mut res = super::Module::new(name, &mut ports, kind.clone());
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

