//! Types and utilities related to FIRRTL statements

pub(crate) mod display;
pub(crate) mod parsers;

#[cfg(test)]
mod tests;

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
pub enum Statement {
    Connection{from: Expression, to: Expression},
    PartialConnection{from: Expression, to: Expression},
    Empty,
    Declaration(Arc<Entity>),
    Invalidate(Expression),
    Attach(Vec<Expression>),
    Conditional{cond: Expression, when: Arc<[Self]>, r#else: Arc<[Self]>},
    Stop{clock: Expression, cond: Expression, code: i64},
    Print{clock: Expression, cond: Expression, msg: Vec<PrintElement>},
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

        match self {
            Self::Connection{from, to}              => writeln!(f, "{}{} <= {}", indent.lock(), to, from),
            Self::PartialConnection{from, to}       => writeln!(f, "{}{} <- {}", indent.lock(), to, from),
            Self::Empty                             => writeln!(f, "{}skip", indent.lock()),
            Self::Declaration(entity)               => display::Entity(entity).fmt(indent, f),
            Self::Invalidate(expr)                  => writeln!(f, "{}{} is invalid", indent.lock(), expr),
            Self::Attach(exprs)                     =>
                writeln!(f, "{}attach({})", indent.lock(), CommaSeparated::from(exprs)),
            Self::Conditional{cond, when, r#else}   => {
                let indent = indent.lock();
                writeln!(f, "{}when {}:", indent, cond)?;
                display::StatementList(when.as_ref()).fmt(&mut indent.sub(), f)?;
                if r#else.len() > 0 {
                    writeln!(f, "{}else:", indent)?;
                    display::StatementList(r#else.as_ref()).fmt(&mut indent.sub(), f)?;
                }
                Ok(())
            },
            Self::Stop{clock, cond, code}           =>
                writeln!(f, "{}stop({}, {}, {})", indent.lock(), clock, cond, code),
            Self::Print{clock, cond, msg}           => writeln!(f,
                "{}printf({}, {}, {}, {})",
                indent.lock(),
                clock,
                cond,
                display::FormatString(msg.as_ref()),
                CommaSeparated::from(msg.iter().filter_map(into_expr)),
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
            return Self::Empty
        }

        let opts: [&dyn Fn(&mut Gen) -> Self; 9] = [
            &|g| {
                let t = types::Type::arbitrary(g);
                Self::Connection{
                    from: expr_with_type(t.clone(), source_flow(g), g),
                    to: expr_with_type(t.clone(), sink_flow(g), g),
                }
            },
            &|g| {
                let t = types::Type::arbitrary(g);
                Self::PartialConnection{
                    from: expr_with_type(t.clone(), source_flow(g), g),
                    to: expr_with_type(t.clone(), sink_flow(g), g),
                }
            },
            &|_| Self::Empty,
            &|g| {
                let e = fn_iter(|| Some(Arbitrary::arbitrary(g)))
                    .find(Entity::is_declarable)
                    .unwrap();
                    Self::Declaration(Arc::new(e))
            },
            &|g| Self::Invalidate(expr_with_type(types::Type::arbitrary(g), expr::Flow::Source, g)),
            &|g| {
                let t = GT::Analog(Arbitrary::arbitrary(g));
                let n = u8::arbitrary(g).saturating_add(1);
                Self::Attach(fn_iter(|| Some(expr_with_type(t.clone(), Arbitrary::arbitrary(g), g)))
                    .take(n as usize)
                    .collect())
            },
            &|g| Self::Conditional {
                cond: expr_with_type(GT::UInt(Some(1)), source_flow(g), g),
                when: vec![Self::Empty].into(), // TODO: sequence
                r#else: vec![].into(), // TODO: sequence
            },
            &|g| Self::Stop {
                clock: expr_with_type(GT::Clock, source_flow(g), g),
                cond: expr_with_type(GT::UInt(Some(1)), source_flow(g), g),
                code: Arbitrary::arbitrary(g),
            },
            &|g| Self::Print {
                clock: expr_with_type(GT::Clock, source_flow(g), g),
                cond: expr_with_type(GT::UInt(Some(1)), source_flow(g), g),
                msg: Arbitrary::arbitrary(g),
            },
        ];

        g.choose(&opts).unwrap()(g)
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

        match self {
            Self::Declaration(entity)               => Box::new(entity.shrink().map(Self::Declaration)),
            Self::Attach(exprs)                     => Box::new(
                bisect(exprs.clone()).into_iter().map(Self::Attach)
            ),
            Self::Conditional{cond, when, r#else}   => {
                let cond = cond.clone();
                let r#else = bisect(r#else.clone().to_vec());

                let res = bisect(when.clone().to_vec())
                    .into_iter()
                    .filter(|v| v.len() > 0)
                    .flat_map(move |w| r#else.clone().into_iter().map(move |e| (w.clone(), e)))
                    .map(move |(w, e)| Self::Conditional{cond: cond.clone(), when: w.clone().into(), r#else: e.into()});
                Box::new(res)
            },
            Self::Print{clock, cond, msg}           => {
                let clock = clock.clone();
                let cond = cond.clone();
                Box::new(msg.shrink().map(move |msg| Self::Print{clock: clock.clone(), cond: cond.clone(), msg}))
            },
            _ => Box::new(std::iter::empty()),
        }
    }
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

        fn field_to_port(field: &types::BundleField) -> module::Port {
            let dir = match field.orientation() {
                types::Orientation::Normal  => module::Direction::Output,
                types::Orientation::Flipped => module::Direction::Input,
            };
            module::Port::new(field.name().clone(), field.r#type().clone(), dir)
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
            &|g| Self::Literal(Arbitrary::arbitrary(g)),
            &|g| Self::Value(expr_with_type(GT::arbitrary(g), source_flow(g), g), Arbitrary::arbitrary(g)),
        ];

        g.choose(&opts).unwrap()(g)
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        match self {
            Self::Literal(s)    => Box::new(s.shrink().map(Self::Literal)),
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

