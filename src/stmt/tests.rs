//! Test related to statements

use std::sync::Arc;

use nom::combinator::all_consuming;
use nom::Finish;

use quickcheck::{Arbitrary, Gen, TestResult, Testable};

use crate::expr::{self, Expression, Reference};
use crate::indentation::{DisplayIndented, Indentation};
use crate::memory::simple::Memory as SimpleMem;
use crate::tests::{Equivalence, Identifier};

use super::{Entity, Kind, Statement, print::PrintElement};


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

    let mut mems: Vec<_> = original
        .declarations()
        .filter_map(|e| if let Entity::SimpleMemPort(m) = e.as_ref() {
            Some(m.memory().clone())
        } else {
            None
        })
        .collect();
    mems.sort_unstable_by_key(|r| r.name().clone());
    if mems.windows(2).any(|p| p[0].name() == p[1].name()) {
        // We depend on memory names to be unique.
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
        |n| mems.binary_search_by_key(&n, |r| r.name()).ok().map(|i| mems[i].clone()),
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
fn parse_stmts(mut base: Indentation, original: Statement) -> Result<TestResult, String> {
    let original = if let Some(stmts) = stmt_with_decls(
        original,
        &mut Default::default(),
        &mut Default::default()
    ) {
        stmts
    } else {
        return Ok(TestResult::discard())
    };

    let mut ports: Vec<_> = original
        .iter()
        .flat_map(stmt_exprs)
        .into_iter()
        .flat_map(Expression::references)
        .filter_map(|e| if let Entity::Port(p) = e.as_ref() { Some(p.clone()) } else { None })
        .collect();
    ports.sort_unstable_by_key(|r| r.name().to_string());
    if ports.windows(2).any(|p| p[0].name() == p[1].name()) {
        // We depend on reference names to be unique.
        return Ok(TestResult::discard())
    }

    let mut mods: Vec<_> = original
        .iter()
        .flat_map(Statement::instantiations)
        .map(|i| i.module().clone())
        .collect();
    mods.sort_unstable_by_key(|r| r.name().to_string());
    if mods.windows(2).any(|p| p[0].name() == p[1].name()) {
        // We depend on module names to be unique.
        return Ok(TestResult::discard())
    }

    let mut buf: String = Default::default();
    original.iter().try_for_each(|s| s.fmt(&mut base, &mut buf)).map_err(|e| e.to_string())?;

    let parser = move |i| super::parsers::stmts(
        |n| ports.binary_search_by_key(&n, |r| r.name()).ok().map(|i| Arc::new(ports[i].clone().into())),
        |_| None,
        |n| mods.binary_search_by_key(&n, |r| r.name()).ok().map(|i| mods[i].clone()),
        i,
        &mut base
    );

    let res = all_consuming(parser)(&buf)
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

    let mut refs: Vec<_> = entity_exprs(&original)
        .into_iter()
        .flat_map(Expression::references)
        .cloned()
        .collect();
    refs.sort_unstable_by_key(|r| r.name().to_string());
    if refs.windows(2).any(|p| p[0].name() == p[1].name()) {
        // We depend on reference names to be unique.
        return Ok(TestResult::discard())
    }

    let mems = if let Entity::SimpleMemPort(m) = &original {
        Some(m.memory().clone())
    } else {
        None
    };

    let module = if let Entity::Instance(m) = &original {
        Some(m.module().clone())
    } else {
        None
    };

    let mut s: String = Default::default();
    super::display::EntityDecl(&original, Default::default())
        .fmt(&mut base, &mut s)
        .map_err(|e| e.to_string())?;

    let parser = move |i| super::parsers::entity_decl(
        |n| refs.binary_search_by_key(&n, |r| r.name()).ok().map(|i| refs[i].clone()),
        |n| mems.clone().filter(|m| m.name().as_ref() == n),
        |n| module.clone().filter(|m| m.name() == n),
        i,
        &mut base
    );

    let res = all_consuming(parser)(&s)
        .finish()
        .map(|(_, parsed)| Equivalence::of((original, None), parsed).result(&mut Gen::new(0)))
        .map_err(|e| e.to_string());
    res
}


#[quickcheck]
fn parse_fmt_string(original: FormatString) -> Result<TestResult, String> {
    use nom::character::complete::char as chr;
    use nom::combinator::map;
    use nom::multi::many1;
    use nom::sequence::tuple;

    use super::parsers;

    use PrintElement as PE;
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


#[quickcheck]
fn parse_optional_name(original: Option<Identifier>) -> Result<Equivalence<Option<Arc<str>>>, String> {
    let s = super::display::OptionalName(original.as_ref().map(AsRef::as_ref)).to_string();
    let res = all_consuming(super::parsers::optional_name)(&s)
        .finish()
        .map(|(_, parsed)| Equivalence::of(original.map(Into::into), parsed))
        .map_err(|e| e.to_string());
    res
}


/// Generate a valid sequence of statements from a given input
///
/// This function takes the given statements and inserts additional
/// ones, making sure all referenced declarable `Entities` are both
/// declared before they are used and have unique names.
///
/// The iteration will stop if extension fails for an item, i.e. the output will
/// potentially only contain a subset of the input.
pub fn stmts_with_decls(statements: impl IntoIterator<Item = Statement>) -> impl Iterator<Item = Statement> {
    let mut entities = Default::default();
    let mut memories = Default::default();

    statements
        .into_iter()
        .map(move |s| stmt_with_decls(s, &mut entities, &mut memories))
        .take_while(Option::is_some)
        .flat_map(|v| v.unwrap_or_default())
}


/// Generate a valid sequence of statements ending with a given statement
///
/// This function prepends the given statement with all declarations necessary
/// for it to be valid. If this is not possible, the function returns `None`.
pub fn stmt_with_decls(
    statement: Statement,
    entities: &mut std::collections::HashMap<String, Arc<Entity>>,
    memories: &mut std::collections::HashMap<Arc<str>, Arc<SimpleMem>>,
) -> Option<Vec<Statement>> {
    use std::collections::hash_map::Entry;

    let mut new_decls = Default::default();

    // Make sure memories used in port declarations are defined
    if let Kind::Declaration(e) = statement.kind() {
        if let Entity::SimpleMemPort(p) = e.as_ref() {
            new_decls = stmt_with_decls(
                Kind::SimpleMemDecl(p.memory().clone()).into(),
                entities,
                memories,
            )?;
        }
    }

    let new_decls = stmt_exprs(&statement)
        .into_iter()
        .flat_map(Expression::references)
        .try_fold(new_decls, |mut d, r| {
            match entities.entry(r.name().into()) {
                Entry::Occupied(e) => if e.get() != r { return None }
                Entry::Vacant(e) => {
                    e.insert(r.clone());
                    if r.is_declarable() {
                        d.extend(stmt_with_decls(Kind::Declaration(r.clone()).into(), entities, memories)?)
                    }
                }
            };
            Some(d)
        });

    match statement.kind() {
        Kind::Declaration(entity) => match entities.entry(entity.name().into()) {
            Entry::Occupied(e) => if e.get() != entity { return None }
            Entry::Vacant(e) => { e.insert(entity.clone()); }
        },
        Kind::SimpleMemDecl(mem)  => match memories.entry(mem.name().clone()) {
            Entry::Occupied(e) => if e.get() != mem { return None }
            Entry::Vacant(e) => { e.insert(mem.clone()); }
        },
        _ => (),
    }

    new_decls.map(|mut v| {
        v.push(statement);
        v
    })
}


/// Retrieve all expressions occuring in a statement
pub fn stmt_exprs(stmt: &Statement) -> Vec<&Expression<Arc<Entity>>> {
    match stmt.as_ref() {
        Kind::Connection{from, to}              => vec![from, to],
        Kind::PartialConnection{from, to}       => vec![from, to],
        Kind::Empty                             => Default::default(),
        Kind::Declaration(entity)               => entity_exprs(entity.as_ref()),
        Kind::SimpleMemDecl(_)                  => Default::default(),
        Kind::Invalidate(expr)                  => vec![expr],
        Kind::Attach(v)                         => v.iter().collect(),
        Kind::Conditional{cond, when, r#else}   => std::iter::once(cond)
            .chain(when.iter().flat_map(stmt_exprs))
            .chain(r#else.iter().flat_map(stmt_exprs))
            .collect(),
        Kind::Stop{clock, cond, ..}             => vec![clock, cond],
        Kind::Print{clock, cond, msg, ..}       => std::iter::once(clock)
            .chain(std::iter::once(cond))
            .chain(msg.iter().filter_map(|p| if let PrintElement::Value(e, _) = p {
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
        Entity::Register(reg)       => std::iter::once(reg.clock())
            .chain(reg.reset_signal())
            .chain(reg.reset_value())
            .collect(),
        Entity::Node{value, ..}     => vec![value],
        Entity::SimpleMemPort(port) => vec![port.address(), port.clock()],
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
    data: Vec<PrintElement>
}

impl From<Vec<PrintElement>> for FormatString {
    fn from(data: Vec<PrintElement>) -> Self {
        Self {data}
    }
}

impl From<FormatString> for Vec<PrintElement> {
    fn from(string: FormatString) -> Self {
        string.data
    }
}

impl Arbitrary for FormatString {
    fn arbitrary(g: &mut Gen) -> Self {
        use expr::tests::{expr_with_type, source_flow};
        use crate::types::GroundType as GT;

        use PrintElement as PE;

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
        use PrintElement as PE;

        let res = self.data.shrink().filter(|v| v.windows(2).all(|e| match e {
            [PE::Literal(..), PE::Literal(..)] => false,
            _ => true,
        })).map(Into::into);
        Box::new(res)
    }
}

