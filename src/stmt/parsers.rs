//! Parsers for statements


use std::sync::Arc;

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{anychar, char as chr};
use nom::combinator::{iterator, map, value, verify};
use nom::multi::{many1, separated_list1};
use nom::sequence::{preceded, tuple};

use crate::expr::parsers::expr;
use crate::indentation::Indentation;
use crate::module::parsers::instance;
use crate::module::Module;
use crate::parsers::{IResult, comma, decimal, identifier, kw, le, lp, op, rp, spaced};
use crate::register::parsers::register;
use crate::types::parsers::r#type;
use crate::memory::parsers::memory;


/// Parser for sequences of statements
pub fn stmts<'i>(
    reference: impl Fn(&str) -> Option<std::sync::Arc<super::Entity>>,
    module: impl Fn(&str) -> Option<std::sync::Arc<Module>> + Copy,
    mut input: &'i str,
    indentation: &'_ mut Indentation,
) -> IResult<'i, Vec<super::Statement>> {
    use crate::expr::Reference;

    let mut res: Vec<super::Statement> = Default::default();
    let mut entities: Vec<Arc<super::Entity>> = Default::default();

    while let Ok((i, stmt)) = stmt(
        |n| entities.iter().find(|e| e.name() == n).cloned().or_else(|| reference(n)),
        module,
        input,
        indentation,
    ) {
        if let super::Statement::Declaration(e) = &stmt {
            entities.push(e.clone());
        }
        res.push(stmt);
        input = i;
    }

    Ok((input, res))
}


/// Parser for individual statements
pub fn stmt<'i>(
    reference: impl Fn(&str) -> Option<std::sync::Arc<super::Entity>> + Copy,
    module: impl Fn(&str) -> Option<std::sync::Arc<Module>> + Copy,
    input: &'i str,
    indentation: &'_ mut Indentation,
) -> IResult<'i, super::Statement> {
    use super::{Statement as S, PrintElement as P};

    let indent = indentation.clone().into_parser();

    let expr = |i| expr(reference, i);

    let (input, (indent, stmt)) = alt((
        map(
            tuple((indent.clone(), &expr, spaced(op("<=")), spaced(&expr), le)),
            |(i, to, _, from, _)| (i, S::Connection{from, to}),
        ),
        map(
            tuple((indent.clone(), &expr, spaced(op("<-")), spaced(&expr), le)),
            |(i, to, _, from, _)| (i, S::PartialConnection{from, to}),
        ),
        map(tuple((indent.clone(), kw("skip"), le)), |(i, ..)| (i, S::Empty)),
        |i| {
            let mut indent = indent.clone().into();
            entity_decl(reference, module, i, &mut indent)
                .map(|(i, e)| (i, (indent, S::Declaration(Arc::new(e)))))
        },
        map(
            tuple((indent.clone(), &expr, spaced(kw("is")), spaced(kw("invalid")), le)),
            |(i, e, ..)| (i, S::Invalidate(e)),
        ),
        map(
            tuple((indent.clone(), kw("attach"), lp, separated_list1(comma, spaced(&expr)), rp, le)),
            |(i, _, _, e, _, _)| (i, S::Attach(e)),
        ),
        |i| {
            use nom::Parser;

            let (i, mut indent) = indent.clone().parse(i)?;
            indented_condition(&reference, module, i, &mut indent)
                .map(|(i, stmt)| (i, (indent, stmt)))
        },
        map(
            tuple((
                indent.clone(),
                kw("stop"),
                lp,
                spaced(&expr),
                comma,
                spaced(&expr),
                comma,
                spaced(decimal),
                rp,
                le,
            )),
            |(i, _, _, clock, _, cond, _, code, ..)| (i, S::Stop{clock, cond, code}),
        ),
        map(
            tuple((
                indent.clone(),
                kw("printf"),
                lp,
                spaced(&expr),
                comma,
                spaced(&expr),
                comma,
                spaced(|i| {
                    let (i, fmt_str) = map(
                        tuple((chr('"'), many1(fmt_string_part), chr('"'))),
                        |(_, p, ..)| p
                    )(i)?;
                    let mut exprs = iterator(i, preceded(spaced(comma), spaced(&expr)));
                    let ps: Vec<_> = fmt_str.into_iter().filter_map(|e| match e {
                        FmtStrPart::Literal(s) => Some(P::Literal(s)),
                        FmtStrPart::FormatSpec(f) => (&mut exprs).next().map(|e| P::Value(e, f)),
                    }).collect();
                    exprs.finish().map(|(i, _)| (i, ps))
                }),
                rp,
                le,
            )),
            |(i, _, _, clock, _, cond, _, msg, ..)| (i, S::Print{clock, cond, msg}),
        ),
    ))(input)?;

    *indentation = indent;

    Ok((input, stmt))
}


/// Parser for conditionals, assuming that the initial indendation was parsed
///
/// This parser will parse a conditional statement. It expects the initial
/// `when` right at the beginning of the input and aussumes that is matches the
/// given indentation.
fn indented_condition<'i>(
    reference: &dyn Fn(&str) -> Option<std::sync::Arc<super::Entity>>,
    module: impl Fn(&str) -> Option<std::sync::Arc<Module>> + Copy,
    input: &'i str,
    indentation: &mut Indentation,
) -> IResult<'i, super::Statement> {
    let (input, cond) = map(
        tuple((kw("when"), spaced(|i| expr(reference, i)), spaced(op(":")), le)),
        |(_, e, ..)| e,
    )(input)?;

    let (input, when) = stmts(&reference, module, input, &mut indentation.sub())?;

    let (input, r#else) = if let Ok((i, _)) = tuple((indentation.clone().parser(), kw("else")))(input) {
        if let Ok((i, _)) = tuple((spaced(op(":")), le))(i) {
            stmts(&reference, module, i, &mut indentation.sub())
        } else {
            map(spaced(|i| indented_condition(reference, module, i, indentation)), |s| vec![s],)(i)
        }?
    } else {
        (input, Default::default())
    };

    Ok((
        input,
        super::Statement::Conditional{cond, when: when.into(), r#else: r#else.into()}
    ))
}


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


/// Parser for a format string part
pub fn fmt_string_part<'i>(
    input: &'i str,
) -> IResult<'i, FmtStrPart> {
    use super::Format as F;

    alt((
        value(FmtStrPart::FormatSpec(F::Binary), tag("%b")),
        value(FmtStrPart::FormatSpec(F::Decimal), tag("%d")),
        value(FmtStrPart::FormatSpec(F::Hexadecimal), tag("%x")),
        map(
            many1(alt((
                value('%', tag("%%")),
                value('\n', tag("\\n")),
                value('\t', tag("\\t")),
                preceded(chr('\\'), anychar),
                verify(anychar, |c| !"%\n\t'\"".contains(*c)),
            ))),
            |v| FmtStrPart::Literal(v.into_iter().collect()),
        )
    ))(input)
}


/// Format string part
///
/// Instances of this type serves as prototypes for `PrintElement`s.
#[derive(Clone, Debug)]
pub enum FmtStrPart {
    Literal(String),
    FormatSpec(super::Format)
}

