//! Test related to expressions

use std::sync::Arc;

use nom::combinator::all_consuming;
use nom::Finish;
use quickcheck::{Arbitrary, Gen};

use crate::tests::{Equivalence, Identifier};
use crate::types;

use super::{Expression, parsers, primitive};


#[quickcheck]
fn parse_expr(original: TypedExpr<Identifier>) -> Result<Equivalence<Expression<Identifier>>, String> {
    let s = original.expr.to_string();
    let res = all_consuming(|i| parsers::expr(|s| Some(s.into()), i))(&s)
        .finish()
        .map(|(_, parsed)| Equivalence::of(original.expr, parsed))
        .map_err(|e| e.to_string());
    res
}


#[quickcheck]
fn expr_typing(expr: TypedExpr<Entity>) -> Result<bool, String> {
    use types::Typed;

    // The type retrieved from the expression may not match the type used to
    // generate the expression perfectly. Widths may differ. However, the types
    // must match structurally.
    expr.expr
        .r#type()
        .map_err(|e| format!("{:?}", e))
        .map(|t| crate::TypeExt::eq(&expr.r#type, &t))
}


/// Helper for expressions preserving the type used for generation
///
/// Expressions are generated from a type, but the `Arbitrary` impl discards the
/// type after generation. This struct, however, preserves that type, allowing
/// additional checks.
#[derive(Clone, Debug)]
struct TypedExpr<R: TypedRef> {
    pub expr: Expression<R>,
    pub r#type: types::Type,
}

impl<R: 'static + TypedRef + Clone> Arbitrary for TypedExpr<R> {
    fn arbitrary(g: &mut Gen) -> Self {
        // The type of the expression may be considerably less complex than the
        // expression itself.
        let r#type: types::Type = Arbitrary::arbitrary(&mut Gen::new(g.size() / 10));
        let expr = expr_with_type(r#type.clone(), g);
        Self {expr, r#type}
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        use std::iter::once;

        match &self.expr {
            Expression::SubField{base, index} => {
                let r#type = vec![
                    types::BundleField::new(index.clone(), self.r#type.clone(), Default::default())
                ].into();
                Box::new(once(Self { expr: base.as_ref().clone(), r#type}))
            },
            Expression::SubIndex{base, ..} => Box::new(once(Self {
                expr: base.as_ref().clone(),
                r#type: types::Type::Vector(Arc::new(self.r#type.clone()), 1)
            })),
            Expression::SubAccess{base, index} => Box::new(vec![
                Self {expr: index.as_ref().clone(), r#type: types::GroundType::UInt(None).into()},
                Self {
                    expr: base.as_ref().clone(),
                    r#type: types::Type::Vector(Arc::new(self.r#type.clone()), 1)
                },
            ].into_iter()),
            Expression::Mux{sel, a, b} => Box::new(vec![
                Self {expr: sel.as_ref().clone(), r#type: types::GroundType::UInt(Some(1)).into()},
                Self {expr: a.as_ref().clone(), r#type: self.r#type.clone()},
                Self {expr: b.as_ref().clone(), r#type: self.r#type.clone()},
            ].into_iter()),
            Expression::ValidIf{sel, value} => Box::new(vec![
                Self {expr: sel.as_ref().clone(), r#type: types::GroundType::UInt(Some(1)).into()},
                Self {expr: value.as_ref().clone(), r#type: self.r#type.clone()},
            ].into_iter()),
            Expression::PrimitiveOp(op) => {
                use types::TypeExt;

                let res = self
                    .r#type
                    .ground_type()
                    .map(|g| shrink_primitive_op(op, g))
                    .unwrap_or_default()
                    .into_iter();
                Box::new(res)
            },
            _ => Box::new(std::iter::empty()),
        }
    }
}


fn shrink_primitive_op<R: TypedRef + Clone>(
    op: &primitive::Operation<R>,
    r#type: types::GroundType,
) -> Vec<TypedExpr<R>> {
    use primitive::Operation as PO;

    let with_width = |e: &Arc<Expression<R>>, w| TypedExpr {
        expr: e.as_ref().clone(),
        r#type: r#type.with_width(w).into()
    };
    let uint = |e: &Arc<Expression<R>>| TypedExpr {
        expr: e.as_ref().clone(),
        r#type: types::GroundType::UInt(None).into()
    };
    let fixed = |e: &Arc<Expression<R>>| TypedExpr {
        expr: e.as_ref().clone(),
        r#type: types::GroundType::Fixed(None, None).into()
    };

    match op {
        PO::Add(l, r)           => vec![with_width(l, None), with_width(r, None)],
        PO::Sub(l, r)           => vec![with_width(l, None), with_width(r, None)],
        PO::Mul(l, r)           => vec![with_width(l, None), with_width(r, None)],
        PO::Div(l, r)           => vec![with_width(l, None), with_width(r, None)],
        PO::Rem(l, r)           => vec![with_width(l, None), with_width(r, None)],
        // Comparisions mask operand types
        PO::Pad(e, _)           => vec![with_width(e, None)],
        // For casts, we only know the target type
        PO::Shl(e, b)           => vec![with_width(e, r#type.width().and_then(|w| w.checked_sub(b.get())))],
        PO::Shr(e, _)           => vec![with_width(e, None)],
        PO::DShl(e, b)          => vec![with_width(e, None), uint(b)],
        PO::DShr(e, b)          => vec![with_width(e, None), uint(b)],
        // Cvt operand can be SInt or UInt
        PO::Neg(e)              => vec![with_width(e, r#type.width().and_then(|w| w.checked_sub(1)))],
        // Not operand can be SInt or UInt
        PO::And(l, r)           => vec![with_width(l, None), with_width(r, None)],
        PO::Or(l, r)            => vec![with_width(l, None), with_width(r, None)],
        PO::Xor(l, r)           => vec![with_width(l, None), with_width(r, None)],
        // Reduction op operand can be SInt or UInt
        PO::Cat(l, r)           => vec![with_width(l, None), with_width(r, None)],
        PO::Bits(e, _, _)       => vec![with_width(e, None)],
        PO::IncPrecision(e, _)  => vec![fixed(e)],
        PO::DecPrecision(e, _)  => vec![fixed(e)],
        PO::SetPrecision(e, _)  => vec![fixed(e)],
        _ => Default::default(),
    }
}


/// Entity to use as a Reference for tests involving typing
///
/// Unlike Identifier, this implements `Typed`, i.e. it can hold a type.
#[derive(Clone, Debug, PartialEq)]
struct Entity {
    name: Identifier,
    r#type: types::Type,
}

impl TypedRef for Entity {
    fn with_type(r#type: types::Type, g: &mut Gen) -> Self {
        Self {name: Arbitrary::arbitrary(g), r#type}
    }
}

impl super::Typed for Entity {
    type Err = ();

    type Type = types::Type;

    fn r#type(&self) -> Result<Self::Type, Self::Err> {
        Ok(self.r#type.clone())
    }
}

impl super::Reference for Entity {
    fn name(&self) -> &str {
        self.name.name()
    }

    fn flow(&self) -> super::Flow {
        self.name.flow()
    }
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

