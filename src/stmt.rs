// Copyright (c) 2021 FZI Forschungszentrum Informatik
// SPDX-License-Identifier: Apache-2.0
//! FIRRTL statements and associated utilities

pub(crate) mod display;
pub(crate) mod parsers;

pub mod context;
pub mod entity;
pub mod print;

#[cfg(test)]
pub mod tests;

use std::fmt;
use std::sync::Arc;

#[cfg(test)]
use quickcheck::{Arbitrary, Gen};

use crate::expr;
use crate::indentation::{DisplayIndented, Indentation};
use crate::info;
use crate::memory::simple::Memory as SimpleMem;
use crate::module;

pub use entity::Entity;


/// FIRRTL statement
#[derive(Clone, Debug, PartialEq)]
pub struct Statement {
    kind: Kind,
    info: Option<String>,
}

impl Statement {
    /// Retrieve all declarations appearing in this statement
    ///
    /// This function retrieves all entities declared in a given statement.
    /// Obviously, a declaration will yield an entity. However, the returned
    /// iterator will also include declarations declared in nested statements,
    /// e.g. inside conditional branches.
    pub fn declarations(&self) -> impl Iterator<Item = &Arc<Entity>> {
        use transiter::AutoTransIter;

        self.trans_iter().filter_map(|s| if let Kind::Declaration(e) = s.as_ref() {
            Some(e)
        } else {
            None
        })
    }

    /// Retrieve all instantiations appearing in this statement
    ///
    /// This function retrieves all [module::Instance]s (declarations) in a
    /// given statement. This includes instantiations in nested statements,
    /// e.g. inside conditional branches.
    pub fn instantiations(&self) -> impl Iterator<Item = &module::Instance> {
        self.declarations().filter_map(|e| if let Entity::Instance(i) = e.as_ref() {
            Some(i)
        } else {
            None
        })
    }

    /// Retrieve the statement [Kind]
    pub fn kind(&self) -> &Kind {
        &self.kind
    }
}

impl From<Kind> for Statement {
    fn from(kind: Kind) -> Self {
        Self {kind, info: Default::default()}
    }
}

impl AsRef<Kind> for Statement {
    fn as_ref(&self) -> &Kind {
        self.kind()
    }
}

impl info::WithInfo for Statement {
    fn info(&self) -> Option<&str> {
        self.info.as_ref().map(AsRef::as_ref)
    }

    fn set_info(&mut self, info: Option<String>) {
        self.info = info
    }
}

impl<'a> transiter::AutoTransIter<&'a Statement> for &'a Statement {
    type RecIter = Vec<Self>;

    fn recurse(item: &Self) -> Self::RecIter {
        if let Kind::Conditional{when, r#else, ..} = item.kind() {
            when.iter().chain(r#else.iter()).collect()
        } else {
            Default::default()
        }
    }
}

impl DisplayIndented for Statement {
    fn fmt<W: fmt::Write>(&self, indent: &mut Indentation, f: &mut W) -> fmt::Result {
        use crate::display::CommaSeparated;
        use crate::info::Info;
        use display::OptionalName;

        fn into_expr(elem: &print::PrintElement) -> Option<&Expression> {
            if let print::PrintElement::Value(expr, _) = elem {
                Some(expr)
            } else {
                None
            }
        }

        fn fmt_indendet_cond(
            cond: &Expression,
            when: &Arc<[Statement]>,
            r#else: &Arc<[Statement]>,
            indent: &mut Indentation,
            info: Info,
            f: &mut impl fmt::Write,
        ) -> fmt::Result {
            writeln!(f, "when {}:{}", cond, info)?;
            display::StatementList(when.as_ref()).fmt(&mut indent.sub(), f)?;

            if let [stmt] = r#else.as_ref() {
                if let Kind::Conditional{cond, when, r#else} = stmt.as_ref() {
                    write!(f, "{}else ", indent.lock())?;
                    return fmt_indendet_cond(cond, when, r#else, indent, Info::of(stmt), f);
                }
            }

            if r#else.len() > 0 {
                writeln!(f, "{}else:", indent.lock())?;
                display::StatementList(r#else.as_ref()).fmt(&mut indent.sub(), f)
            } else {
                Ok(())
            }
        }

        let info = Info::of(self);

        match self.as_ref() {
            Kind::Connection{from, to}              =>
                writeln!(f, "{}{} <= {}{}", indent.lock(), to, from, info),
            Kind::PartialConnection{from, to}       =>
                writeln!(f, "{}{} <- {}{}", indent.lock(), to, from, info),
            Kind::Empty                             => writeln!(f, "{}skip{}", indent.lock(), info),
            Kind::Declaration(entity)               => display::EntityDecl(entity, info).fmt(indent, f),
            Kind::SimpleMemDecl(mem)                => writeln!(f, "{}{}{}", indent.lock(), mem, info),
            Kind::Invalidate(expr)                  => writeln!(f, "{}{} is invalid", indent.lock(), expr),
            Kind::Attach(exprs)                     =>
                writeln!(f, "{}attach({}){}", indent.lock(), CommaSeparated::from(exprs), info),
            Kind::Conditional{cond, when, r#else}   => {
                write!(f, "{}", indent.lock())?;
                fmt_indendet_cond(cond, when, r#else, indent, info, f)
            },
            Kind::Stop{name, clock, cond, code}     => writeln!(f,
                "{}stop({}, {}, {}){}{}",
                indent.lock(),
                clock,
                cond,
                code,
                OptionalName::from(name.as_ref().map(AsRef::as_ref)),
                info,
            ),
            Kind::Print{name, clock, cond, msg}     => writeln!(f,
                "{}printf({}, {}, {}{}){}{}",
                indent.lock(),
                clock,
                cond,
                display::FormatString(msg.as_ref()),
                CommaSeparated::from(msg.iter().filter_map(into_expr)).with_preceding(),
                OptionalName::from(name.as_ref().map(AsRef::as_ref)),
                info,
            ),
        }
    }
}

#[cfg(test)]
impl Arbitrary for Statement {
    fn arbitrary(g: &mut Gen) -> Self {
        use std::iter::from_fn as fn_iter;

        use crate::tests::Identifier;
        use expr::tests::{expr_with_type, sink_flow, source_flow};
        use crate::types::{GroundType as GT, Type};

        if g.size() == 0 {
            return Kind::Empty.into()
        }

        let opts: [&dyn Fn(&mut Gen) -> Kind; 10] = [
            &|g| {
                let t = Type::arbitrary(g);
                Kind::Connection{
                    from: expr_with_type(t.clone(), source_flow(g), g),
                    to: expr_with_type(t.clone(), sink_flow(g), g),
                }
            },
            &|g| {
                let t = Type::arbitrary(g);
                Kind::PartialConnection{
                    from: expr_with_type(t.clone(), source_flow(g), g),
                    to: expr_with_type(t.clone(), sink_flow(g), g),
                }
            },
            &|_| Kind::Empty,
            &|g| {
                let e = fn_iter(|| Some(Arbitrary::arbitrary(g)))
                    .find(Entity::is_declarable)
                    .unwrap();
                    Kind::Declaration(Arc::new(e))
            },
            &|g| Kind::SimpleMemDecl(Arbitrary::arbitrary(g)),
            &|g| Kind::Invalidate(expr_with_type(Type::arbitrary(g), expr::Flow::Source, g)),
            &|g| {
                let t = GT::Analog(Arbitrary::arbitrary(g));
                let n = u8::arbitrary(g).saturating_add(1);
                Kind::Attach(fn_iter(|| Some(expr_with_type(t.clone(), Arbitrary::arbitrary(g), g)))
                    .take(n as usize)
                    .collect())
            },
            &|g| Kind::Conditional {
                cond: expr_with_type(GT::UInt(Some(1)), source_flow(g), g),
                when: tests::stmt_list(u8::arbitrary(g).saturating_add(1), g).into(),
                r#else: tests::stmt_list(u8::arbitrary(g), g).into(),
            },
            &|g| Kind::Stop {
                name: Option::<Identifier>::arbitrary(g).map(Into::into),
                clock: expr_with_type(GT::Clock, source_flow(g), g),
                cond: expr_with_type(GT::UInt(Some(1)), source_flow(g), g),
                code: Arbitrary::arbitrary(g),
            },
            &|g| Kind::Print {
                name: Option::<Identifier>::arbitrary(g).map(Into::into),
                clock: expr_with_type(GT::Clock, source_flow(g), g),
                cond: expr_with_type(GT::UInt(Some(1)), source_flow(g), g),
                msg: tests::FormatString::arbitrary(g).into(),
            },
        ];

        // We want to reduce the effective generation size in order to keep
        // statements at a reasonable size. At the same time, we don't
        // necessarily want to take care of all our various generators to cope
        // with a size of `0`. Hence, we need to make sure the size is non-zero.
        g.choose(&opts).unwrap()(&mut Gen::new(std::cmp::max(g.size() / 5, 1))).into()
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        use crate::tests::Identifier;

        fn bisect<T: Clone>(mut v: Vec<T>) -> Vec<Vec<T>> {
            if v.len() > 1 {
                let right = v.split_off(v.len() / 2);
                vec![v.into(), right.into()]
            } else {
                Default::default()
            }
        }

        match self.as_ref() {
            Kind::Declaration(entity)               => Box::new(
                entity.shrink().map(Kind::Declaration).map(Into::into)
            ),
            Kind::Attach(exprs)                     => Box::new(
                bisect(exprs.clone()).into_iter().filter(|v| !v.is_empty()).map(Kind::Attach).map(Into::into)
            ),
            Kind::Conditional{cond, when, r#else}   => {
                let cond = cond.clone();
                let e = r#else.to_vec();

                let res = when.to_vec().shrink()
                    .filter(|v| !v.is_empty())
                    .flat_map(move |w| e.shrink().map(move |e| (w.clone(), e)))
                    .map(move |(w, e)| Kind::Conditional{
                        cond: cond.clone(),
                        when: w.clone().into(),
                        r#else: e.into(),
                    }.into());
                Box::new(when.to_vec().into_iter().chain(r#else.to_vec()).chain(res))
            },
            Kind::Stop{name, clock, cond, code}     => {
                let clock = clock.clone();
                let cond = cond.clone();
                let code = *code;

                let res = name
                    .as_ref()
                    .map(|n| Identifier::from(n.as_ref()))
                    .shrink()
                    .map(move |name| Kind::Stop{
                        name: name.map(Into::into),
                        clock: clock.clone(),
                        cond: cond.clone(),
                        code,
                    }.into());
                Box::new(res)
            }
            Kind::Print{name, clock, cond, msg}     => {
                let clock = clock.clone();
                let cond = cond.clone();

                let res = (
                    name.as_ref().map(|n| Identifier::from(n.as_ref())),
                    tests::FormatString::from(msg.clone()),
                ).shrink().map(move |(n, m)| Kind::Print{
                    name: n.map(Into::into),
                    clock: clock.clone(),
                    cond: cond.clone(),
                    msg: m.into(),
                }.into());
                Box::new(res)
            },
            _ => Box::new(std::iter::empty()),
        }
    }
}


/// [Statement] kind
#[derive(Clone, Debug, PartialEq)]
pub enum Kind {
    Connection{from: Expression, to: Expression},
    PartialConnection{from: Expression, to: Expression},
    Empty,
    Declaration(Arc<Entity>),
    SimpleMemDecl(Arc<SimpleMem>),
    Invalidate(Expression),
    Attach(Vec<Expression>),
    Conditional{cond: Expression, when: Arc<[Statement]>, r#else: Arc<[Statement]>},
    Stop{name: Option<Arc<str>>, clock: Expression, cond: Expression, code: i64},
    Print{name: Option<Arc<str>>, clock: Expression, cond: Expression, msg: Vec<print::PrintElement>},
}


/// Expression type suitable for [Statement]s
type Expression = expr::Expression<Arc<Entity>>;

