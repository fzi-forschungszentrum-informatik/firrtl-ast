//! Types and utilities related to FIRRTL statements

pub(crate) mod display;
pub(crate) mod parsers;

use std::fmt;
use std::sync::Arc;

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


/// An element in a print statement
#[derive(Clone, Debug, PartialEq)]
pub enum PrintElement {
    Literal(String),
    Value(Expression, Format),
}


/// Foramt specifier for print statements
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Format {Binary, Decimal, Hexadecimal}

