//! Test related to statements

use std::sync::Arc;

use nom::combinator::all_consuming;
use nom::Finish;

use quickcheck::{Gen, TestResult, Testable};

use crate::expr::{Expression, Reference};
use crate::indentation::{DisplayIndented, Indentation};
use crate::tests::Equivalence;

use super::{Entity, Statement};


#[quickcheck]
fn parse_stmt(mut base: Indentation, original: Statement) -> Result<TestResult, String> {
    // We depend on reference and module names to be unique. If they are not,
    // the set of names will be smaller than the corresponding list.
    let refs: Vec<_> = stmt_exprs(&original).into_iter().flat_map(Expression::references).cloned().collect();
    if refs.iter().map(|r| r.name()).collect::<std::collections::HashSet<_>>().len() != refs.len() {
        return Ok(TestResult::discard())
    }

    let mods = stmt_modules(&original);
    if mods.iter().map(|r| r.name()).collect::<std::collections::HashSet<_>>().len() != refs.len() {
        return Ok(TestResult::discard())
    }

    let mut s: String = Default::default();
    original.fmt(&mut base, &mut s).map_err(|e| e.to_string())?;

    let parser = move |i| super::parsers::stmt(
        |n| refs.iter().find(|r| r.name() == n).cloned(),
        |n| mods.iter().find(|m| m.name() == n).cloned(),
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


/// Retrieve all expressions occuring in a statement
fn stmt_exprs(stmt: &Statement) -> Vec<&Expression<Arc<Entity>>> {
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


/// Retrieve all modules instanciated in a statement
fn stmt_modules(stmt: &Statement) -> Vec<Arc<crate::module::Module>> {
    match stmt {
        Statement::Declaration(m) => if let Entity::Instance(m) = m.as_ref() {
            vec![m.module().clone()]
        } else {
            Default::default()
        },
        Statement::Conditional{when, r#else, ..} => when
            .iter()
            .flat_map(stmt_modules)
            .chain(r#else.iter().flat_map(stmt_modules))
            .collect(),
        _ => Default::default(),
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

