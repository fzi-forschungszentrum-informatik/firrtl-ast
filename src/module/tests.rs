//! Tests related to modules

use std::sync::Arc;

use nom::Finish;
use nom::combinator::all_consuming;

use quickcheck::{Gen, TestResult, Testable};

use crate::indentation::{DisplayIndented, Indentation};
use crate::stmt;
use crate::tests::Equivalence;

use super::{Direction, Instance, Module, Port, parsers};


#[quickcheck]
fn parse_module(mut base: Indentation, original: Module) -> Result<TestResult, String> {
    let mut s: String = Default::default();
    original.fmt(&mut base, &mut s).map_err(|e| e.to_string())?;

    let mut mods: Vec<_> = original.referenced_modules().cloned().collect();
    mods.sort_unstable_by_key(|r| r.name().to_string());
    if mods.windows(2).any(|p| p[0].name() == p[1].name()) {
        // We depend on module names to be unique.
        return Ok(TestResult::discard())
    }

    let res = all_consuming(
        |i| parsers::module(
            |n| mods.binary_search_by_key(&n, |r| r.name()).ok().map(|i| mods[i].clone()),
            i,
            &mut base,
        )
    )(&s)
        .finish()
        .map(|(_, parsed)| Equivalence::of(original, parsed).result(&mut Gen::new(0)))
        .map_err(|e| e.to_string());
    res
}


#[quickcheck]
fn parse_instance(original: Instance) -> Result<Equivalence<Instance>, String> {
    let s = original.to_string();

    let m = original.module().clone();
    let lookup = move |n: &str| if n == m.name() {
        Some(m.clone())
    } else {
        None
    };

    let res = all_consuming(|i| parsers::instance(&lookup, i))(&s)
        .finish()
        .map(|(_, parsed)| Equivalence::of(original, parsed))
        .map_err(|e| e.to_string());
    res
}


#[quickcheck]
fn parse_port(original: Port) -> Result<Equivalence<Port>, String> {
    let s = original.to_string();

    let res = all_consuming(|i| parsers::port(i))(&s)
        .finish()
        .map(|(_, parsed)| Equivalence::of(original, parsed))
        .map_err(|e| e.to_string());
    res
}


#[quickcheck]
fn parse_direction(original: Direction) -> Result<Equivalence<Direction>, String> {
    let s = original.to_string();
    let res = all_consuming(parsers::direction)(&s)
        .finish()
        .map(|(_, parsed)| Equivalence::of(original, parsed))
        .map_err(|e| e.to_string());
    res
}


/// Create a regular module from the given statements
///
/// The statements are extended with necessary declarations. From those
/// statements, the function constructs a regular module with the
/// required ports.
///
/// The module returned may not contain all statements fed to the function. The
/// consumption is halted if a given statement we would require conflicting
/// declarations or if the number of ports would exceeed the given maximum.
pub fn module_with_stmts(
    name: Arc<str>,
    stmts: impl IntoIterator<Item = stmt::Statement>,
    max_ports: usize,
) -> Module {
    use stmt::tests::{stmt_exprs, stmt_with_decls};

    use crate::expr::Expression;

    let mut entities: std::collections::HashMap<String, Arc<stmt::Entity>> = Default::default();
    let mut ports: std::collections::HashMap<String, Arc<Port>> = Default::default();

    let stmts = stmts
        .into_iter()
        .map(|s| stmt_with_decls(s, &mut entities))
        .take_while(Option::is_some)
        .flat_map(|v| v.unwrap_or_default())
        .take_while(|s| {
            ports.extend(
                stmt_exprs(s)
                    .into_iter()
                    .flat_map(Expression::references)
                    .filter_map(|e| if let stmt::Entity::Port(p) = e.as_ref() { Some(p) } else { None })
                    .map(|p| (p.name().into(), p.clone()))
            );
            ports.len() <= max_ports
        })
        .collect();

    let mut module = Module::new(name, ports.into_iter().map(|(_, v)| v), super::Kind::Regular);
    module.statements_mut().map(|s| *s = stmts);
    module
}

