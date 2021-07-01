//! Test related to statements

use std::sync::Arc;

use nom::combinator::all_consuming;
use nom::Finish;

use quickcheck::{Arbitrary, Gen, TestResult, Testable};

use crate::expr::{self, Expression, Reference};
use crate::indentation::{DisplayIndented, Indentation};
use crate::tests::Equivalence;

use super::{Entity, Statement};


#[quickcheck]
fn parse_stmt(mut base: Indentation, original: Statement) -> Result<TestResult, String> {
    let mut refs: Vec<_> = stmt_exprs(&original)
        .into_iter()
        .flat_map(Expression::references)
        .cloned()
        .collect();
    refs.sort_unstable_by_key(|r| r.name().to_string());
    if refs.windows(2).any(|p| p[0].name() == p[1].name()) {
        // We depend on reference names to be unique.
        return Ok(TestResult::discard())
    }

    let mut mods: Vec<_> = original.instantiations().map(|i| i.module().clone()).collect();
    mods.sort_unstable_by_key(|r| r.name().to_string());
    if mods.windows(2).any(|p| p[0].name() == p[1].name()) {
        // We depend on module names to be unique.
        return Ok(TestResult::discard())
    }

    let mut s: String = Default::default();
    original.fmt(&mut base, &mut s).map_err(|e| e.to_string())?;

    let parser = move |i| super::parsers::stmt(
        |n| refs.binary_search_by_key(&n, |r| r.name()).ok().map(|i| refs[i].clone()),
        |n| mods.binary_search_by_key(&n, |r| r.name()).ok().map(|i| mods[i].clone()),
        i,
        &mut base
    );

    let res = all_consuming(parser)(&s)
        .finish()
        .map(|(_, parsed)| Equivalence::of(original, parsed).result(&mut Gen::new(0)))
        .map_err(|e| e.to_string());
    res
}


#[quickcheck]
fn parse_entity(mut base: Indentation, original: Entity) -> Result<TestResult, String> {
    if !original.is_declarable() {
        return Ok(TestResult::discard())
    }

    // We depend on reference names to be unique. If they are not, the set of
    // names will be smaller than the list of references.
    let refs: Vec<_> = entity_exprs(&original)
        .into_iter()
        .flat_map(Expression::references)
        .cloned()
        .collect();
    if refs.iter().map(|r| r.name()).collect::<std::collections::HashSet<_>>().len() != refs.len() {
        return Ok(TestResult::discard())
    }

    let module: Option<_> = if let Entity::Instance(m) = &original {
        Some(m.module().clone())
    } else {
        None
    };

    let mut s: String = Default::default();
    super::display::Entity(&original).fmt(&mut base, &mut s).map_err(|e| e.to_string())?;

    let parser = move |i| super::parsers::entity_decl(
        |n| refs.iter().find(|r| r.name() == n).cloned(),
        |n| module.clone().and_then(|m| if m.name() == n { Some(m) } else { None }),
        i,
        &mut base
    );

    let res = all_consuming(parser)(&s)
        .finish()
        .map(|(_, parsed)| Equivalence::of(original, parsed).result(&mut Gen::new(0)))
        .map_err(|e| e.to_string());
    res
}


#[quickcheck]
fn parse_fmt_string(original: FormatString) -> Result<TestResult, String> {
    use nom::character::complete::char as chr;
    use nom::combinator::map;
    use nom::multi::many1;
    use nom::sequence::tuple;

    use super::{PrintElement as PE, parsers};
    use parsers::FmtStrPart as FSP;

    let original: Vec<_> = original.into();
    let s = super::display::FormatString(original.as_ref()).to_string();
    let parsed = all_consuming(map(tuple((chr('"'), many1(parsers::fmt_string_part), chr('"'))), |(_, p, ..)| p))(&s)
        .finish()
        .map_err(|e| e.to_string())
        .map(|(_, p)| p)?;

    let identical = original.into_iter().zip(parsed).all(|i| match i {
        (PE::Literal(o),    FSP::Literal(p))    => o == p,
        (PE::Value(_, o),   FSP::FormatSpec(p)) => o == p,
        _ => false,
    });

    Ok(TestResult::from_bool(identical))
}


/// Retrieve all expressions occuring in a statement
pub fn stmt_exprs(stmt: &Statement) -> Vec<&Expression<Arc<Entity>>> {
    match stmt {
        Statement::Connection{from, to}             => vec![from, to],
        Statement::PartialConnection{from, to}      => vec![from, to],
        Statement::Empty                            => Default::default(),
        Statement::Declaration(entity)              => entity_exprs(entity.as_ref()),
        Statement::Invalidate(expr)                 => vec![expr],
        Statement::Attach(v)                        => v.iter().collect(),
        Statement::Conditional{cond, when, r#else}  => std::iter::once(cond)
            .chain(when.iter().flat_map(stmt_exprs))
            .chain(r#else.iter().flat_map(stmt_exprs))
            .collect(),
        Statement::Stop{clock, cond, ..}            => vec![clock, cond],
        Statement::Print{clock, cond, msg}          => std::iter::once(clock)
            .chain(std::iter::once(cond))
            .chain(msg.iter().filter_map(|p| if let super::PrintElement::Value(e, _) = p {
                Some(e)
            } else {
                None
            }))
            .collect(),
    }
}


/// Retrieve all expressions occuring in an entity decl
fn entity_exprs(entity: &Entity) -> Vec<&Expression<Arc<Entity>>> {
    match entity {
        Entity::Register(reg)   => std::iter::once(reg.clock())
            .chain(reg.reset_signal())
            .chain(reg.reset_value())
            .collect(),
        Entity::Node{value, ..} => vec![value],
        _ => Default::default(),
    }
}


/// Generate a list of statements with the given length
pub fn stmt_list(len: impl Into<usize>, g: &mut Gen) -> Vec<super::Statement> {
    let len = len.into();
    let mut g = Gen::new(g.size() / std::cmp::max(len, 1));
    std::iter::from_fn(|| Some(quickcheck::Arbitrary::arbitrary(&mut g))).take(len).collect()
}


/// Utility for generating format strings
///
/// This utility generates format strings, making sure it never has two adjacent
/// `PrintElement::Literal`s and at least one `PrintElement::Value`.
#[derive(Clone, Debug)]
pub struct FormatString {
    data: Vec<super::PrintElement>
}

impl From<Vec<super::PrintElement>> for FormatString {
    fn from(data: Vec<super::PrintElement>) -> Self {
        Self {data}
    }
}

impl From<FormatString> for Vec<super::PrintElement> {
    fn from(string: FormatString) -> Self {
        string.data
    }
}

impl Arbitrary for FormatString {
    fn arbitrary(g: &mut Gen) -> Self {
        use expr::tests::{expr_with_type, source_flow};
        use crate::types::GroundType as GT;

        use super::PrintElement as PE;

        let mut data = vec![Arbitrary::arbitrary(g)];

        let len = u8::arbitrary(g) as usize;
        let mut g = Gen::new(g.size() / std::cmp::max(len, 1));

        (0..len).for_each(|_| {
            data.push(match data.last().unwrap() {
                PE::Literal(..) => PE::Value(
                    expr_with_type(GT::arbitrary(&mut g), source_flow(&mut g), &mut g),
                    Arbitrary::arbitrary(&mut g)
                ),
                PE::Value(..)   => Arbitrary::arbitrary(&mut g),
            })
        });

        data.into()
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        use super::PrintElement as PE;

        let res = self.data.shrink().filter(|v| v.windows(2).all(|e| match e {
            [PE::Literal(..), PE::Literal(..)] => false,
            _ => true,
        })).map(Into::into);
        Box::new(res)
    }
}

