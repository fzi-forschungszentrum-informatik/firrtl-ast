// Copyright (c) 2021 FZI Forschungszentrum Informatik
// SPDX-License-Identifier: Apache-2.0
//! Parsers for statements


use std::sync::Arc;

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{anychar, char as chr};
use nom::combinator::{iterator, map, opt, value, verify};
use nom::multi::{many1, separated_list1};
use nom::sequence::{preceded, tuple};

use crate::expr::parsers::expr;
use crate::indentation::Indentation;
use crate::info::{WithInfo, parse as info};
use crate::memory::parsers::{memory, register, simple_mem, simple_mem_port};
use crate::module::parsers::instance;
use crate::parsers::{IResult, comma, decimal, identifier, kw, le, lp, op, rp, spaced};
use crate::types::parsers::r#type;

use super::{context::Context, print};


/// Parser for sequences of statements
pub fn stmts<'i>(
    mut ctx: impl Context,
    mut input: &'i str,
    indentation: &'_ mut Indentation,
) -> IResult<'i, Vec<super::Statement>> {
    let mut res: Vec<super::Statement> = Default::default();

    while let Ok((i, stmt)) = stmt(&mut ctx, input, indentation) {
        match stmt.as_ref() {
            super::Kind::Declaration(e)     => ctx.add_entity(e.clone()),
            super::Kind::SimpleMemDecl(m)   => ctx.add_memory(m.clone()),
            _ => (),
        }
        res.push(stmt);
        input = i;
    }

    Ok((input, res))
}


/// Parser for individual statements
pub fn stmt<'i>(
    ctx: &'_ mut impl Context,
    input: &'i str,
    indentation: &'_ mut Indentation,
) -> IResult<'i, super::Statement> {
    use super::{Kind, Statement as S};
    use print::PrintElement as P;

    let indent = indentation.clone().into_parser();

    let expr = |i| expr(|n| ctx.entity(n), i);

    let res = alt((
        map(
            tuple((indent.clone(), &expr, spaced(op("<=")), spaced(&expr), info, le)),
            |(i, to, _, from, info, _)| (i, S::from(Kind::Connection{from, to}).with_info(info)),
        ),
        map(
            tuple((indent.clone(), &expr, spaced(op("<-")), spaced(&expr), info, le)),
            |(i, to, _, from, info, _)| (i, S::from(Kind::PartialConnection{from, to}).with_info(info)),
        ),
        map(
            tuple((indent.clone(), kw("skip"), info, le)),
            |(i, _, info, ..)| (i, S::from(Kind::Empty).with_info(info))),
        |i| {
            let mut indent = indent.clone().into();
            entity_decl(ctx, i, &mut indent)
                .map(|(i, (e, info))| (i, (indent, S::from(Kind::Declaration(Arc::new(e))).with_info(info))))
        },
        map(
            tuple((indent.clone(), simple_mem, info, le)),
            |(i, mem, info, _)| (i, S::from(Kind::SimpleMemDecl(Arc::new(mem))).with_info(info)),
        ),
        map(
            tuple((indent.clone(), &expr, spaced(kw("is")), spaced(kw("invalid")), info, le)),
            |(i, e, .., info, _)| (i, S::from(Kind::Invalidate(e)).with_info(info)),
        ),
        map(
            tuple((indent.clone(), kw("attach"), lp, separated_list1(comma, spaced(&expr)), rp, info, le)),
            |(i, _, _, e, _, info, _)| (i, S::from(Kind::Attach(e)).with_info(info)),
        ),
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
                optional_name,
                info,
                le,
            )),
            |(i, _, _, clock, _, cond, _, code, _, name, info, ..)|
                (i, S::from(Kind::Stop{name, clock, cond, code}).with_info(info)),
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
                optional_name,
                info,
                le,
            )),
            |(i, _, _, clock, _, cond, _, msg, _, name, info, ..)|
                (i, S::from(Kind::Print{name, clock, cond, msg}).with_info(info)),
        ),
    ))(input);

    let (input, (indent, stmt)) = res.or_else(|_| {
        use nom::Parser;

        let (i, mut indent) = indent.clone().parse(input)?;
        indented_condition(ctx, i, &mut indent).map(|(i, stmt)| (i, (indent, stmt)))
    })?;

    *indentation = indent;

    Ok((input, stmt))
}


/// Parser for conditionals, assuming that the initial indendation was parsed
///
/// This parser will parse a conditional statement. It expects the initial
/// `when` right at the beginning of the input and aussumes that is matches the
/// given indentation.
fn indented_condition<'i>(
    ctx: &'_ mut impl Context,
    input: &'i str,
    indentation: &mut Indentation,
) -> IResult<'i, super::Statement> {
    let (input, (cond, when_info)) = map(
        tuple((kw("when"), spaced(|i| expr(|n| ctx.entity(n), i)), spaced(op(":")), info, le)),
        |(_, e, _, info, ..)| (e, info),
    )(input)?;

    let (input, when) = stmts(ctx.sub(), input, &mut indentation.sub())?;

    let (input, r#else) = if let Ok((i, _)) = tuple((indentation.clone().parser(), kw("else")))(input) {
        if let Ok((i, _)) = tuple((spaced(op(":")), info, le))(i) {
            stmts(ctx.sub(), i, &mut indentation.sub())
        } else {
            map(spaced(|i| indented_condition(&mut ctx.sub(), i, indentation)), |s| vec![s],)(i)
        }?
    } else {
        (input, Default::default())
    };

    let cond = super::Kind::Conditional{cond, when: when.into(), r#else: r#else.into()};
    Ok((input, super::Statement::from(cond).with_info(when_info)))
}


/// Parser for entity declarations
pub fn entity_decl<'i>(
    ctx: &'_ impl Context,
    input: &'i str,
    indentation: &'_ mut Indentation,
) -> IResult<'i, (super::Entity, Option<String>)> {
    use nom::Parser;

    let indent = indentation.clone().into_parser();
    let ident = |i| spaced(identifier).parse(i);

    let (input, (indent, entity, info)) = alt((
        map(
            tuple((indent.clone(), kw("wire"), &ident, spaced(op(":")), spaced(r#type), info, le)),
            |(i, _, n, _, r#type, info, _)| (i, super::Entity::Wire{name: n.into(), r#type}, info)
        ),
        map(
            tuple((indent.clone(), |i| register(|n| ctx.entity(n), i), info, le)),
            |(i, r, info, _)| (i, r.into(), info)
        ),
        map(
            tuple((
                indent.clone(),
                kw("node"),
                &ident,
                spaced(op("=")),
                spaced(|i| expr(|n| ctx.entity(n), i)),
                info,
                le
            )),
            |(i, _, n, _, value, info, _)| (i, super::Entity::Node{name: n.into(), value}, info)
        ),
        |i| {
            let mut indent = Into::into(indent.clone());
            memory(i, &mut indent).map(|(i, (m, info))| (i, (indent, m.into(), info)))
        },
        map(
            tuple((indent.clone(), |i| simple_mem_port(|n| ctx.memory(n), |n| ctx.entity(n), i), info, le)),
            |(i, r, info, _)| (i, r.into(), info)
        ),
        map(
            tuple((indent.clone(), |i| instance(|n| ctx.module(n), i), info, le)),
            |(i, inst, info, _)| (i, inst.into(), info)
        ),
    ))(input)?;

    *indentation = indent;

    Ok((input, (entity, info)))
}


/// Parser for a format string part
pub fn fmt_string_part<'i>(
    input: &'i str,
) -> IResult<'i, FmtStrPart> {
    use print::Format as F;

    alt((
        value(FmtStrPart::FormatSpec(F::Binary), tag("%b")),
        value(FmtStrPart::FormatSpec(F::Decimal), tag("%d")),
        value(FmtStrPart::FormatSpec(F::Hexadecimal), tag("%x")),
        value(FmtStrPart::FormatSpec(F::Character), tag("%c")),
        map(
            many1(alt((
                value('%', tag("%%")),
                value('\n', tag("\\n")),
                value('\t', tag("\\t")),
                preceded(chr('\\'), anychar),
                verify(anychar, |c| !"%\n\t\"".contains(*c)),
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
    FormatSpec(print::Format)
}


/// Parse optional name
pub fn optional_name<'i>(
    input: &'i str,
) -> IResult<'i, Option<Arc<str>>> {
    opt(map(tuple((spaced(op(":")), spaced(identifier))), |(_, n)| n.into()))(input)
}

