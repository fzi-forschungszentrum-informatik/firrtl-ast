//! Types and utilities related to FIRRTL statements

pub(crate) mod display;
pub(crate) mod parsers;

#[cfg(test)]
pub mod tests;

use std::fmt;
use std::sync::Arc;

#[cfg(test)]
use quickcheck::{Arbitrary, Gen};

use crate::expr;
use crate::indentation::{DisplayIndented, Indentation};
use crate::memory::Memory;
use crate::module;
use crate::register::Register;
use crate::types;


/// FIRRTL statement
#[derive(Clone, Debug, PartialEq)]
pub struct Statement {
    kind: Kind,
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
    /// This function retrieves all module instantiations (declarations) in a
    /// given statement. This includes instantiations in nested statements,
    /// e.g. inside conditional branches.
    pub fn instantiations(&self) -> impl Iterator<Item = &module::Instance> {
        self.declarations().filter_map(|e| if let Entity::Instance(i) = e.as_ref() {
            Some(i)
        } else {
            None
        })
    }

    /// Retrieve the statement kind
    pub fn kind(&self) -> &Kind {
        &self.kind
    }
}

impl From<Kind> for Statement {
    fn from(kind: Kind) -> Self {
        Self {kind}
    }
}

impl AsRef<Kind> for Statement {
    fn as_ref(&self) -> &Kind {
        self.kind()
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

        fn into_expr(elem: &PrintElement) -> Option<&Expression> {
            if let PrintElement::Value(expr, _) = elem {
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
            f: &mut impl fmt::Write,
        ) -> fmt::Result {
            writeln!(f, "when {}:", cond)?;
            display::StatementList(when.as_ref()).fmt(&mut indent.sub(), f)?;

            if let [stmt] = r#else.as_ref() {
                if let Kind::Conditional{cond, when, r#else} = stmt.as_ref() {
                    write!(f, "{}else ", indent.lock())?;
                    return fmt_indendet_cond(cond, when, r#else, indent, f);
                }
            }

            if r#else.len() > 0 {
                writeln!(f, "{}else:", indent.lock())?;
                display::StatementList(r#else.as_ref()).fmt(&mut indent.sub(), f)
            } else {
                Ok(())
            }
        }

        match self.as_ref() {
            Kind::Connection{from, to}              => writeln!(f, "{}{} <= {}", indent.lock(), to, from),
            Kind::PartialConnection{from, to}       => writeln!(f, "{}{} <- {}", indent.lock(), to, from),
            Kind::Empty                             => writeln!(f, "{}skip", indent.lock()),
            Kind::Declaration(entity)               => display::Entity(entity).fmt(indent, f),
            Kind::Invalidate(expr)                  => writeln!(f, "{}{} is invalid", indent.lock(), expr),
            Kind::Attach(exprs)                     =>
                writeln!(f, "{}attach({})", indent.lock(), CommaSeparated::from(exprs)),
            Kind::Conditional{cond, when, r#else}   => {
                write!(f, "{}", indent.lock())?;
                fmt_indendet_cond(cond, when, r#else, indent, f)
            },
            Kind::Stop{clock, cond, code}           =>
                writeln!(f, "{}stop({}, {}, {})", indent.lock(), clock, cond, code),
            Kind::Print{clock, cond, msg}           => writeln!(f,
                "{}printf({}, {}, {}{})",
                indent.lock(),
                clock,
                cond,
                display::FormatString(msg.as_ref()),
                CommaSeparated::from(msg.iter().filter_map(into_expr)).with_preceding(),
            ),
        }
    }
}

#[cfg(test)]
impl Arbitrary for Statement {
    fn arbitrary(g: &mut Gen) -> Self {
        use std::iter::from_fn as fn_iter;

        use expr::tests::{expr_with_type, sink_flow, source_flow};
        use types::GroundType as GT;

        if g.size() == 0 {
            return Kind::Empty.into()
        }

        let opts: [&dyn Fn(&mut Gen) -> Kind; 9] = [
            &|g| {
                let t = types::Type::arbitrary(g);
                Kind::Connection{
                    from: expr_with_type(t.clone(), source_flow(g), g),
                    to: expr_with_type(t.clone(), sink_flow(g), g),
                }
            },
            &|g| {
                let t = types::Type::arbitrary(g);
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
            &|g| Kind::Invalidate(expr_with_type(types::Type::arbitrary(g), expr::Flow::Source, g)),
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
                clock: expr_with_type(GT::Clock, source_flow(g), g),
                cond: expr_with_type(GT::UInt(Some(1)), source_flow(g), g),
                code: Arbitrary::arbitrary(g),
            },
            &|g| Kind::Print {
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
            Kind::Print{clock, cond, msg}           => {
                let clock = clock.clone();
                let cond = cond.clone();
                let res = tests::FormatString::from(msg.clone())
                    .shrink()
                    .map(move |msg| Kind::Print{
                        clock: clock.clone(),
                        cond: cond.clone(),
                        msg: msg.into(),
                    }.into());
                Box::new(res)
            },
            _ => Box::new(std::iter::empty()),
        }
    }
}


/// Statement kind
#[derive(Clone, Debug, PartialEq)]
pub enum Kind {
    Connection{from: Expression, to: Expression},
    PartialConnection{from: Expression, to: Expression},
    Empty,
    Declaration(Arc<Entity>),
    Invalidate(Expression),
    Attach(Vec<Expression>),
    Conditional{cond: Expression, when: Arc<[Statement]>, r#else: Arc<[Statement]>},
    Stop{clock: Expression, cond: Expression, code: i64},
    Print{clock: Expression, cond: Expression, msg: Vec<PrintElement>},
}


/// Expression type suitable for statements
type Expression = expr::Expression<Arc<Entity>>;


/// Referencable entity
///
/// FIRRTL defines several entities which may be referenced inside an
/// expression.
#[derive(Clone, Debug, PartialEq)]
pub enum Entity {
    Port(Arc<module::Port>),
    Wire{name: Arc<str>, r#type: types::Type},
    Register(Register<Arc<Self>>),
    Node{name: Arc<str>, value: expr::Expression<Arc<Self>>},
    Memory(Memory),
    Instance(module::Instance),
}

impl Entity {
    /// Checks whether this entity can be declared via a statement
    ///
    /// Returns true if the entity can be declared, which will be the case for
    /// most entities. Note that `Port`s cannot be declared.
    pub fn is_declarable(&self) -> bool {
        match self {
            Self::Port(..)  => false,
            _ => true,
        }
    }
}

impl From<Arc<module::Port>> for Entity {
    fn from(port: Arc<module::Port>) -> Self {
        Self::Port(port)
    }
}

impl From<Register<Arc<Entity>>> for Entity {
    fn from(register: Register<Arc<Entity>>) -> Self {
        Self::Register(register)
    }
}

impl From<Memory> for Entity {
    fn from(mem: Memory) -> Self {
        Self::Memory(mem)
    }
}

impl From<module::Instance> for Entity {
    fn from(inst: module::Instance) -> Self {
        Self::Instance(inst)
    }
}

impl expr::Reference for Arc<Entity> {
    fn name(&self) -> &str {
        match self.as_ref() {
            Entity::Port(port)      => port.name(),
            Entity::Wire{name, ..}  => name.as_ref(),
            Entity::Register(reg)   => reg.name(),
            Entity::Node{name, ..}  => name.as_ref(),
            Entity::Memory(mem)     => mem.name(),
            Entity::Instance(inst)  => inst.name(),
        }
    }

    fn flow(&self) -> expr::Flow {
        match self.as_ref() {
            Entity::Port(port)      => port.flow(),
            Entity::Wire{..}        => expr::Flow::Duplex,
            Entity::Register(reg)   => reg.flow(),
            Entity::Node{..}        => expr::Flow::Source,
            Entity::Memory(mem)     => mem.flow(),
            Entity::Instance(inst)  => inst.flow(),
        }
    }
}

impl types::Typed for Arc<Entity> {
    type Err = Self;

    type Type = types::Type;

    fn r#type(&self) -> Result<Self::Type, Self::Err> {
        match self.as_ref() {
            Entity::Port(port)          => Ok(port.r#type().clone()),
            Entity::Wire{r#type, ..}    => Ok(r#type.clone()),
            Entity::Register(reg)       => reg.r#type().map_err(|_| self.clone()),
            Entity::Node{value, ..}     => value.r#type().map_err(|_| self.clone()),
            Entity::Memory(mem)         => mem.r#type().map_err(|_| self.clone()),
            Entity::Instance(inst)      => inst.r#type().map_err(|_| self.clone()),
        }
    }
}

#[cfg(test)]
impl expr::tests::TypedRef for Arc<Entity> {
    fn with_type(r#type: types::Type, flow: expr::Flow, g: &mut Gen) -> Self {
        use crate::tests::Identifier;

        use expr::tests::{expr_with_type, source_flow};

        fn field_to_port(field: &types::BundleField) -> Arc<module::Port> {
            let dir = match field.orientation() {
                types::Orientation::Normal  => module::Direction::Output,
                types::Orientation::Flipped => module::Direction::Input,
            };
            Arc::new(module::Port::new(field.name().clone(), field.r#type().clone(), dir))
        }

        let mut opts: Vec<&dyn Fn(Identifier, types::Type, &mut Gen) -> Entity> = match flow {
            expr::Flow::Source => vec![
                &|n, t, _| Arc::new(module::Port::new(n.to_string(), t, module::Direction::Input)).into(),
                &|n, t, g| Entity::Node{name: n.into(), value: expr_with_type(t, source_flow(g), g)},
            ],
            expr::Flow::Sink => vec![
                &|n, t, _| Arc::new(module::Port::new(n.to_string(), t, module::Direction::Output)).into(),
            ],
            expr::Flow::Duplex => vec![
                &|n, t, _| Entity::Wire{name: n.into(), r#type: t},
                &|n, t, g| Register::new(n, t, expr_with_type(types::GroundType::Clock, source_flow(g), g))
                    .into(),
            ],
        };

        if let (types::Type::Bundle(_), expr::Flow::Source) = (&r#type, flow) {
            opts.push(&|n, t, g| {
                let m = module::Module::new(
                    Identifier::arbitrary(g).into(),
                    t.fields().unwrap().map(field_to_port),
                    Arbitrary::arbitrary(g),
                );
                module::Instance::new(n, Arc::new(m)).into()
            })
        }

        Arc::new(g.choose(opts.as_ref()).unwrap()(Identifier::arbitrary(g), r#type, g))
    }
}

#[cfg(test)]
impl Arbitrary for Entity {
    fn arbitrary(g: &mut Gen) -> Self {
        use crate::tests::Identifier;

        use expr::tests::{expr_with_type, source_flow};

        let opts: [&dyn Fn(&mut Gen) -> Entity; 6] = [
            &|g| Arc::new(module::Port::arbitrary(g)).into(),
            &|g| Entity::Wire{name: Identifier::arbitrary(g).into(), r#type: Arbitrary::arbitrary(g)},
            &|g| Register::arbitrary(g).into(),
            &|g| Entity::Node{
                name: Identifier::arbitrary(g).into(),
                value: expr_with_type(types::Type::arbitrary(g), source_flow(g), g)
            },
            &|g| Memory::arbitrary(g).into(),
            &|g| module::Instance::arbitrary(g).into(),
        ];

        g.choose(&opts).unwrap()(g)
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        use crate::tests::Identifier;

        match self {
            Self::Port(port)            => Box::new(port.shrink().map(Into::into)),
            Self::Wire{name, r#type}    => {
                let n = name.clone();
                let t = r#type.clone();
                let res = Identifier::from(name.as_ref())
                    .shrink()
                    .map(move |n| Self::Wire{name: n.into(), r#type: t.clone()})
                    .chain(r#type.shrink().map(move |r#type| Self::Wire{name: n.clone(), r#type}));
                Box::new(res)
            },
            Self::Register(reg)         => Box::new(reg.shrink().map(Into::into)),
            Self::Node{name, value}     => {
                let v = value.clone();
                let res = Identifier::from(name.as_ref())
                    .shrink()
                    .map(move |n| Self::Node{name: n.into(), value: v.clone()});
                Box::new(res)
            },
            Self::Memory(mem)           => Box::new(mem.shrink().map(Into::into)),
            Self::Instance(inst)        => Box::new(inst.shrink().map(Into::into)),
        }
    }
}


/// An element in a print statement
#[derive(Clone, Debug, PartialEq)]
pub enum PrintElement {
    Literal(String),
    Value(Expression, Format),
}

#[cfg(test)]
impl Arbitrary for PrintElement {
    fn arbitrary(g: &mut Gen) -> Self {
        use expr::tests::{expr_with_type, source_flow};
        use types::GroundType as GT;

        let opts: [&dyn Fn(&mut Gen) -> Self; 2] = [
            &|g| Self::Literal(crate::tests::ASCII::arbitrary(g).to_string()),
            &|g| Self::Value(expr_with_type(GT::arbitrary(g), source_flow(g), g), Arbitrary::arbitrary(g)),
        ];

        if g.size() > 0 {
            g.choose(&opts).unwrap()(g)
        } else {
            Self::Literal(" ".to_string())
        }
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        use crate::tests::ASCII;

        match self {
            Self::Literal(s)    => Box::new(
                ASCII::from(s.clone()).shrink().map(|s| Self::Literal(s.to_string()))
            ),
            Self::Value(_, _)   => Box::new(std::iter::empty()),
        }
    }
}


/// Foramt specifier for print statements
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Format {Binary, Decimal, Hexadecimal}

#[cfg(test)]
impl Arbitrary for Format {
    fn arbitrary(g: &mut Gen) -> Self {
        g.choose(&[Self::Binary, Self::Decimal, Self::Hexadecimal]).unwrap().clone()
    }
}

