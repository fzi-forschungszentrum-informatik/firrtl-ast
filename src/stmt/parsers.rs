//! Parsers for statements


use nom::branch::alt;
use nom::combinator::map;
use nom::sequence::tuple;

use crate::expr::parsers::expr;
use crate::indentation::Indentation;
use crate::module::parsers::instance;
use crate::module::Module;
use crate::parsers::{IResult, identifier, kw, le, op, spaced};
use crate::register::parsers::register;
use crate::types::parsers::r#type;
use crate::memory::parsers::memory;


/// Parser for entity declarations
pub fn entity_decl<'i>(
    reference: impl Fn(&str) -> Option<std::sync::Arc<super::Entity>> + Copy,
    module: impl Fn(&str) -> Option<std::sync::Arc<Module>> + Copy,
    input: &'i str,
    indentation: &'_ mut Indentation,
) -> IResult<'i, super::Entity> {
    use nom::Parser;

    let indent = indentation.clone().into_parser();
    let ident = |i| spaced(identifier).parse(i);

    let (input, (indent, entity)) = alt((
        map(
            tuple((indent.clone(), kw("wire"), &ident, spaced(op(":")), spaced(r#type), le)),
            |(i, _, n, _, r#type, _)| (i, super::Entity::Wire{name: n.into(), r#type})
        ),
        map(
            tuple((indent.clone(), |i| register(reference, i), le)),
            |(i, r, _)| (i, r.into())
        ),
        map(
            tuple((
                indent.clone(),
                kw("node"),
                &ident,
                spaced(op("=")),
                spaced(|i| expr(reference, i)),
                le
            )),
            |(i, _, n, _, value, _)| (i, super::Entity::Node{name: n.into(), value})
        ),
        |i| {
            let mut indent = Into::into(indent.clone());
            memory(i, &mut indent).map(|(i, m)| (i, (indent, m.into())))
        },
        map(
            tuple((indent.clone(), |i| instance(module, i), le)),
            |(i, inst, _)| (i, inst.into())
        ),
    ))(input)?;

    *indentation = indent;

    Ok((input, entity))
}

