//! Test related to expressions

use std::sync::Arc;

use nom::combinator::all_consuming;
use nom::Finish;
use quickcheck::{Arbitrary, Gen};

use crate::tests::{Equivalence, Identifier};
use crate::types;

use super::{Expression, parsers, primitive};


#[quickcheck]
fn parse_expr(original: Expression<Identifier>) -> Result<Equivalence<Expression<Identifier>>, String> {
    let s = original.to_string();
    let res = all_consuming(|i| parsers::expr(|s| Some(s.into()), i))(&s)
        .finish()
        .map(|(_, parsed)| Equivalence::of(original, parsed))
        .map_err(|e| e.to_string());
    res
}


#[quickcheck]
fn parse_primitive_op(
    original: primitive::Operation<Identifier>
) -> Result<Equivalence<primitive::Operation<Identifier>>, String> {
    let s = original.to_string();
    let res = all_consuming(|i| parsers::primitive_op(|s| Some(s.into()), i))(&s)
        .finish()
        .map(|(_, parsed)| Equivalence::of(original, parsed))
        .map_err(|e| e.to_string());
    res
}


/// Utility trait for generating references with a given type
pub trait TypedRef: super::Reference {
    /// Generate a reference with the given type
    fn with_type(r#type: types::Type, g: &mut Gen) -> Self;
}

impl TypedRef for Identifier {
    fn with_type(_type: types::Type, g: &mut Gen) -> Self {
        Arbitrary::arbitrary(g)
    }
}


/// Generate an expression which could have the given type
pub fn expr_with_type<R, T>(r#type: T, g: &mut Gen) -> Expression<R>
where R: TypedRef,
      T: Into<types::Type> + types::TypeExt + Clone,
{
    use types::{GroundType as GT};

    if g.size() <= 0 {
        return Expression::Reference(TypedRef::with_type(r#type.into(), g))
    }

    let mut opts: Vec<&dyn Fn(T, &mut Gen) -> Expression<R>> = vec![
        &|t, g| Expression::Reference(TypedRef::with_type(t.into(), g)),
        &|t, g| {
            let index: Arc<str> = Identifier::arbitrary(g).into();
            let bundle = bundle_with_field(t.into(), index.clone(), g);
            Expression::SubField{base: expr(bundle, g), index}
        },
        &|t, g| Expression::SubIndex{
            base: expr(vec_with_base(t.into(), g), g),
            index: Arbitrary::arbitrary(g),
        },
        &|t, g| Expression::SubAccess{
            base: expr(vec_with_base(t.into(), g), g),
            index: expr(GT::UInt(Arbitrary::arbitrary(g)), g),
        },
        &|t, g| Expression::Mux{
            sel: expr(GT::UInt(Some(1)), g),
            a: expr(t.clone(), g),
            b: expr(t.clone(), g),
        },
        &|t, g| Expression::ValidIf{sel: expr(GT::UInt(Some(1)), g), value: expr(t, g)},
    ];

    if let Some(r#type) = r#type.ground_type() {
        opts.push(&|t, g| match t.ground_type().expect("Not a ground type") {
            GT::UInt(width) => Expression::UIntLiteral{
                value: Arbitrary::arbitrary(g),
                width: width.unwrap_or_else(|| Arbitrary::arbitrary(g))
            },
            GT::SInt(width) => Expression::SIntLiteral{
                value: Arbitrary::arbitrary(g),
                width: width.unwrap_or_else(|| Arbitrary::arbitrary(g))
            },
            _ => Expression::Reference(TypedRef::with_type(t.into(), g)),
        });
        match r#type {
            GT::Analog(_) => (),
            _ => opts.push(
                &|t, g| primitive_op_with_type(t.ground_type().expect("Not a ground type"), g).into()
            ),
        }
    }

    g.choose(opts.as_ref()).unwrap()(r#type, &mut Gen::new(g.size() / 2))
}


/// Generate a primitive operation which could have the given type
pub fn primitive_op_with_type<R>(r#type: types::GroundType, g: &mut Gen) -> primitive::Operation<R>
where R: TypedRef,
{
    use types::GroundType as GT;
    use primitive::Operation as PO;

    fn uint_or_sint(w: types::BitWidth, g: &mut Gen) -> GT {
        *g.choose(&[GT::UInt(w), GT::SInt(w)]).unwrap()
    }

    fn uint_sint_or_fixed(w: types::BitWidth, g: &mut Gen) -> GT {
        *g.choose(&[GT::UInt(w), GT::SInt(w), GT::Fixed(w, None)]).unwrap()
    }

    let mut opts: Vec<&dyn Fn(GT, &mut Gen) -> PO<R>> = Default::default();

    // Primitives returning UInt or SInt or Fixed
    if match r#type { GT::UInt(..) | GT::SInt(..) | GT::Fixed(..) => true, _ => false } {
        opts.push(&|t, g| PO::Mul(expr(t.with_width(None), g), expr(t.with_width(None), g)));

        opts.push(&|t, g| PO::Pad(expr(t.with_width(None), g), Arbitrary::arbitrary(g)));

        opts.push(&|t, g| PO::Shl(expr(t.with_width(None), g), Arbitrary::arbitrary(g)));
        opts.push(&|t, g| PO::Shr(expr(t.with_width(None), g), Arbitrary::arbitrary(g)));
        opts.push(&|t, g| PO::DShl(expr(t.with_width(None), g), expr(GT::UInt(None), g)));
        opts.push(&|t, g| PO::DShr(expr(t.with_width(None), g), expr(GT::UInt(None), g)));

        if r#type.width() != Some(0) {
            opts.push(&|t, g| PO::Add(expr(t.with_width(None), g), expr(t.with_width(None), g)));
            opts.push(&|t, g| PO::Sub(expr(t.with_width(None), g), expr(t.with_width(None), g)));
        }
    }

    // Primitives returning UInt or SInt
    if match r#type { GT::UInt(..) | GT::SInt(..) => true, _ => false } {
        opts.push(&|t, g| PO::Div(expr(t.with_width(None), g), expr(t.with_width(None), g)));
        opts.push(&|t, g| PO::Rem(expr(t.with_width(None), g), expr(t.with_width(None), g)));

        opts.push(&|t, g| PO::Cast(expr(uint_sint_or_fixed(None, g), g), t.with_width(None)));

        if r#type.width().unwrap_or(1) == 1 {
            opts.push(&|t, g| PO::Cast(expr(GT::Clock, g), t.with_width(None)));
        }
    }

    // Primitives returning a specific variant only
    match r#type {
        GT::UInt(width) => {
            opts.push(&|_, g| PO::Not(expr(uint_or_sint(None, g), g)));
            opts.push(&|_, g| {
                let t = uint_or_sint(None, g);
                PO::And(expr(t, g), expr(t, g))
            });
            opts.push(&|_, g| {
                let t = uint_or_sint(None, g);
                PO::Or(expr(t, g), expr(t, g))
            });
            opts.push(&|_, g| {
                let t = uint_or_sint(None, g);
                PO::Xor(expr(t, g), expr(t, g))
            });

            opts.push(&|_, g| {
                let t = uint_or_sint(None, g);
                PO::Cat(expr(t, g), expr(t, g))
            });

            if width != Some(0) {
                opts.push(&|_, g| PO::Bits(
                    expr(uint_sint_or_fixed(None, g), g),
                    Some(Arbitrary::arbitrary(g)),
                    None,
                ));
                opts.push(&|_, g| PO::Bits(
                    expr(uint_sint_or_fixed(None, g), g),
                    None,
                    Some(Arbitrary::arbitrary(g)),
                ));
                opts.push(&|_, g| {
                    let a = Arbitrary::arbitrary(g);
                    let b = Arbitrary::arbitrary(g);
                    PO::Bits(
                        expr(uint_sint_or_fixed(None, g), g),
                        Some(std::cmp::min(a, b)),
                        Some(std::cmp::max(a, b)),
                    )
                });
            }

            if width.unwrap_or(1) == 1 {
                opts.push(&|_, g| {
                    let t = uint_sint_or_fixed(None, g);
                    PO::Lt(expr(t, g), expr(t, g))
                });
                opts.push(&|_, g| {
                    let t = uint_sint_or_fixed(None, g);
                    PO::LEq(expr(t, g), expr(t, g))
                });
                opts.push(&|_, g| {
                    let t = uint_sint_or_fixed(None, g);
                    PO::Gt(expr(t, g), expr(t, g))
                });
                opts.push(&|_, g| {
                    let t = uint_sint_or_fixed(None, g);
                    PO::GEq(expr(t, g), expr(t, g))
                });
                opts.push(&|_, g| {
                    let t = uint_sint_or_fixed(None, g);
                    PO::Eq(expr(t, g), expr(t, g))
                });
                opts.push(&|_, g| {
                    let t = uint_sint_or_fixed(None, g);
                    PO::NEq(expr(t, g), expr(t, g))
                });

                opts.push(&|_, g| PO::AndReduce(expr(uint_or_sint(None, g), g)));
                opts.push(&|_, g| PO::OrReduce(expr(uint_or_sint(None, g), g)));
                opts.push(&|_, g| PO::XorReduce(expr(uint_or_sint(None, g), g)));
            }
        },
        GT::SInt(width) if width != Some(0) => {
            opts.push(&|_, g| PO::Cvt(expr(uint_or_sint(None, g), g)));
            opts.push(&|_, g| PO::Neg(expr(uint_or_sint(None, g), g)));
        },
        GT::Fixed(..) => {
            opts.push(&|_, g| PO::IncPrecision(expr(GT::Fixed(None, None), g), Arbitrary::arbitrary(g)));
            opts.push(&|_, g| PO::DecPrecision(expr(GT::Fixed(None, None), g), Arbitrary::arbitrary(g)));
            opts.push(&|t, g| if let GT::Fixed(_, p) = t {
                PO::SetPrecision(expr(GT::Fixed(None, p), g), p.unwrap_or_else(|| Arbitrary::arbitrary(g)))
            } else {
                panic!("Expected 'fixed' type")
            });

            opts.push(&|t, g| if let GT::Fixed(_, p) = t {
                PO::Cast(
                    expr(uint_sint_or_fixed(None, g), g),
                    GT::Fixed(None, p.or_else(|| Some(Arbitrary::arbitrary(g))))
                )
            } else {
                panic!("Expected 'fixed' type")
            });
        },
        GT::Clock => opts.push(&|t, g| PO::Cast(
            expr(*g.choose(&[GT::UInt(None), GT::SInt(None), GT::Fixed(None, None), GT::Clock]).unwrap(), g),
            t.with_width(None)
        )),
        _ => (),
    }

    g.choose(opts.as_ref()).expect("No suitable primitive operation")(r#type, &mut Gen::new(g.size() / 2))
}


/// Generate an expression which could have the given type, wrapped in an Arc
fn expr<R, T>(r#type: T, g: &mut Gen) -> Arc<Expression<R>>
where R: TypedRef,
      T: Into<types::Type> + types::TypeExt + Clone,
{
    Arc::new(expr_with_type(r#type, g))
}


/// Generate a bundle type with a field constructed from the given type and name
fn bundle_with_field(r#type: types::Type, name: Arc<str>, g: &mut Gen) -> types::Type {
    let mut fields = types::bundle_fields(u8::arbitrary(g) as usize, g);
    let field = types::BundleField::new(name.clone(), r#type, Arbitrary::arbitrary(g));
    fields.insert(name, field);
    fields.into()
}


/// Generate a vector type with the given base/item type
fn vec_with_base(r#type: types::Type, g: &mut Gen) -> types::Type {
    types::Type::Vector(Arc::new(r#type), Arbitrary::arbitrary(g))
}

