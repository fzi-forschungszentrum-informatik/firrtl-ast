//! Test related to statements

use std::sync::Arc;

use nom::combinator::all_consuming;
use nom::Finish;

use quickcheck::{Gen, TestResult, Testable};

use crate::expr::{Expression, Reference};
use crate::indentation::{DisplayIndented, Indentation};
use crate::tests::Equivalence;

use super::Entity;


#[quickcheck]
fn parse_entity(mut base: Indentation, original: Entity) -> Result<TestResult, String> {
    if !original.is_declarable() {
        return Ok(TestResult::discard())
    }

    // We depend on reference names to be unique. If they are not, the set of
    // names will be smaller than the list of references.
    let refs: Vec<_> = entity_exprs(&original).into_iter().flat_map(Expression::references).collect();
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

