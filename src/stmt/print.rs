//! Utilities related to printf statments

use super::Expression;

#[cfg(test)]
use quickcheck::{Arbitrary, Gen};


/// An element in a print statement
#[derive(Clone, Debug, PartialEq)]
pub enum PrintElement {
    Literal(String),
    Value(Expression, Format),
}

#[cfg(test)]
impl Arbitrary for PrintElement {
    fn arbitrary(g: &mut Gen) -> Self {
        use crate::expr::tests::{expr_with_type, source_flow};
        use crate::types::GroundType as GT;

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

