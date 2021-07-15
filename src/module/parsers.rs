//! Parsers for modules and related items

use std::sync::Arc;

use nom::branch::alt;
use nom::character::complete::char as chr;
use nom::combinator::{iterator, map, value};
use nom::multi::many0;
use nom::sequence::tuple;

use crate::error::{ParseError, convert_error};
use crate::indentation::Indentation;
use crate::info::{WithInfo, parse as parse_info};
use crate::parsers::{IResult, decimal, float, identifier, kw, le, op, spaced, unquoted_string};
use crate::stmt::parsers::stmts as parse_stmts;
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
    let (input, (name, mut kind, info)) = map(
        tuple((indentation.parser(), kind, spaced(identifier), spaced(op(":")), parse_info, le)),
        |(_, kind, name, _, info, ..)| (name.into(), kind, info)
    )(input)?;

    let mut indentation = indentation.sub();

    let (input, ports) = many0(
        map(tuple((indentation.parser(), port, le)), |(_, p, ..)| Arc::new(p))
    )(input)?;

    let input = match &mut kind {
        super::Kind::Regular{stmts} => {
            let (input, s) = parse_stmts(
                |n| ports
                    .iter()
                    .find(|p| p.name.as_ref() == n)
                    .map(|p| Arc::new(p.clone().into())),
                |_| None,
                module,
                input,
                &mut indentation,
            )?;

            *stmts = s;
            input
        },
        super::Kind::External{defname, params} => {
            let (input, n) = nom::combinator::opt(
                map(
                    tuple((indentation.parser(), kw("defname"), spaced(op("=")), spaced(identifier), le)),
                    |(.., n, _)| n.into()
                )
            )(input)?;
            *defname = n;

            let mut param_iter = iterator(
                input,
                map(
                    tuple((
                        indentation.parser(),
                        kw("parameter"),
                        spaced(identifier),
                        spaced(op("=")),
                        spaced(param_value),
                        le,
                    )),
                    |(.., k, _, v, _)| (k.into(), v)
                ),
            );
            params.extend(&mut param_iter);
            param_iter.finish()?.0
        },
    };

    Ok((input, super::Module::new(name, ports, kind).with_info(info)))
}


/// Parse a module kind
pub fn kind<'i>(input: &str) -> IResult<super::Kind> {
    alt((
        map(kw("module"), |_| super::Kind::empty_regular()),
        map(kw("extmodule"), |_| super::Kind::empty_external()),
    ))(input)
}


/// Parse a parameter value
pub fn param_value(input: &str) -> IResult<super::ParamValue> {
    use super::ParamValue as PV;

    alt((
        map(float, PV::Double),
        map(decimal, PV::Int),
        map(
            tuple((chr('"'), |i| unquoted_string(i, &['\n', '\t', '"']), chr('"'))),
            |(_, s, _)| PV::String(s.into())
        ),
        map(
            tuple((chr('\''), |i| unquoted_string(i, &['\n', '\t', '\'']), chr('\''))),
            |(_, s, _)| PV::String(s.into())
        ),
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

