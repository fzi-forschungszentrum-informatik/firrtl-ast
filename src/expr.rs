//! Datatypes and utilities specific to expressions

mod primitive;

use std::sync::Arc;


/// A FIRRTL expression
#[derive(Clone, Debug, PartialEq)]
pub enum Expression<R: Reference> {
    /// An UInt literal
    UIntLiteral{value: u128, width: u16},
    /// An SInt literal
    SIntLiteral{value: i128, width: u16},
    /// A referernce expression
    Reference(R),
    /// A sub-field expression
    SubField{base: Arc<Expression<R>>, index: Arc<str>},
    /// A sub-index expression
    SubIndex{base: Arc<Expression<R>>, index: u16},
    /// A sub-access expression
    SubAccess{base: Arc<Expression<R>>, index: Arc<Expression<R>>},
    /// A multiplexer expression
    Mux{sel: Arc<Expression<R>>, a: Arc<Expression<R>>, b: Arc<Expression<R>>},
    /// A valid-if expression
    ValidIf{sel: Arc<Expression<R>>, value: Arc<Expression<R>>},
    /// A primitive operation
    PrimitiveOp(primitive::Operation<R>),
}


/// A reference to a named entity
pub trait Reference {
    /// Retrieve the name of the referenced entity
    fn name(&self) -> &str;
}

