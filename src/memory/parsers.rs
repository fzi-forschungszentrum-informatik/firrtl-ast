//! Parsers related to memory elements

use nom::branch::alt;
use nom::combinator::{iterator, map, map_opt, opt, value};
use nom::sequence::tuple;

use crate::expr::{Reference, parsers::expr};
use crate::indentation::Indentation;
use crate::parsers::{self, IResult, comma, decimal, identifier, kw, le, lp, op, rp, spaced};
use crate::types::Type;
use crate::types::parsers::r#type;
use crate::info::parse as info;

use super::{common, mem, simple};


/// Parse a Memory
pub fn memory<'i>(
    input: &'i str,
    indentation: &'_ mut Indentation
) -> IResult<'i, (super::Memory, Option<String>)> {
    use nom::error::{ErrorKind as EK, ParseError};

    let (input, (name, info)) = map(
        tuple((indentation.parser(), kw("mem"), spaced(identifier), spaced(op(":")), info, le)),
        |(_, _, name, _, info, ..)| (name, info)
    )(input)?;

    let mut indentation = indentation.sub();
    let mut entries = iterator(input, map(tuple((indentation.parser(), entry, le)), |(_, e, _)| e));

    let mut data_type: Option<Type> = Default::default();
    let mut depth: Option<mem::Depth> = Default::default();
    let mut read_latency: Option<mem::Latency> = Default::default();
    let mut write_latency: Option<mem::Latency> = Default::default();
    let mut ports: Vec<mem::Port> = Default::default();
    let mut ruw: super::ReadUnderWrite = Default::default();
    (&mut entries).for_each(|e| match e {
        Entry::DataType(t)      => data_type = Some(t),
        Entry::Depth(v)         => depth = Some(v),
        Entry::Port(p)          => ports.push(p),
        Entry::ReadLatency(v)   => read_latency = Some(v),
        Entry::WriteLatency(v)  => write_latency = Some(v),
        Entry::RUW(v)           => ruw = v,
    });

    let mut res = mem::Memory::new(
        name,
        data_type.ok_or_else(|| nom::Err::Error(parsers::Error::from_error_kind(input, EK::Permutation)))?,
        depth.ok_or_else(|| nom::Err::Error(parsers::Error::from_error_kind(input, EK::Permutation)))?,
    ).with_read_under_write(ruw);

    res.add_ports(ports);
    if let Some(v) = read_latency {
        res = res.with_read_latency(v);
    }
    if let Some(v) = write_latency {
        res = res.with_write_latency(v);
    }

    entries.finish().map(|(i, _)| (i, (res, info)))
}


/// Parse a simple memory
pub fn simple_mem(input: &str) -> IResult<simple::Memory> {
    use nom::sequence::preceded;

    use simple::Kind;

    #[derive(Copy, Clone, Debug)]
    enum K {Cmem, Smem}

    let (input, (k, name, _, r#type)) = tuple((
        alt((value(K::Cmem, kw("cmem")), value(K::Smem, kw("smem")))),
        spaced(identifier),
        spaced(op(":")),
        spaced(r#type),
    ))(input)?;

    let (input, kind) = match k {
        K::Cmem => (input, Kind::Combinatory),
        K::Smem => map(opt(preceded(comma, spaced(ruw))), |ruw| Kind::Sequential(ruw))(input)?,
    };
    Ok((input, super::simple::Memory::new(name, r#type, kind)))
}


/// Parse a simple memory port
pub fn simple_mem_port<'i, R: Reference + Clone>(
    memory: impl Fn(&str) -> Option<std::sync::Arc<simple::Memory>> + Copy,
    reference: impl Fn(&str) -> Option<R> + Copy,
    input: &'i str
) -> IResult<'i, simple::Port<R>> {
    use common::PortDir as D;

    map(
        tuple((
            alt((
                value(Some(D::Read),        kw("read")),
                value(Some(D::Write),       kw("write")),
                value(Some(D::ReadWrite),   kw("rdwr")),
                value(None,                 kw("infer")),
            )),
            spaced(kw("mport")),
            spaced(identifier),
            spaced(op("=")),
            map_opt(spaced(identifier), memory),
            spaced(op("[")),
            spaced(|i| expr(reference, i)),
            spaced(op("]")),
            spaced(opt(op(","))),
            spaced(|i| expr(reference, i)),
        )),
        |(dir, _, name, _, mem, _, addr, _, _, clock)| simple::Port::new(name, mem, dir, addr, clock)
    )(input)
}


/// Parse a register definition
pub fn register<'i, R: Reference + Clone>(
    reference: impl Fn(&str) -> Option<R> + Copy,
    input: &'i str
) -> IResult<'i, super::Register<R>> {
    use nom::Parser;

    let expr = |i| spaced(|i| expr(reference, i)).parse(i);

    let reset = map(
        tuple((lp, spaced(kw("reset")), spaced(op("=>")), lp, &expr, comma, &expr, rp, rp)),
        |(.., sig, _, val, _, _)| (sig, val)
    );

    let res = map(
        tuple((
            kw("reg"),
            spaced(identifier),
            spaced(op(":")),
            spaced(r#type),
            comma,
            &expr,
            opt(spaced(map(tuple((kw("with"), spaced(op(":")), spaced(reset))), |(.., r)| r)))
        )),
        |(_, name, _, r#type, _, clock, reset)| super::Register::new(name, r#type, clock)
            .with_optional_reset(reset)
    )(input);
    res
}


enum Entry {
    DataType(Type),
    Depth(mem::Depth),
    Port(mem::Port),
    ReadLatency(mem::Latency),
    WriteLatency(mem::Latency),
    RUW(super::ReadUnderWrite),
}


fn entry<'i>(input: &'i str) -> IResult<'i, Entry> {
    alt((
        map(tuple((kw("data-type"), arrow, spaced(r#type))), |(.., t)| Entry::DataType(t)),
        map(tuple((kw("depth"), arrow, spaced(decimal))), |(.., v)| Entry::Depth(v)),
        map(
            tuple((kw("reader"), arrow, spaced(identifier))),
            |(.., n)| Entry::Port(mem::Port {name: n.into(), dir: super::PortDir::Read})
        ),
        map(
            tuple((kw("writer"), arrow, spaced(identifier))),
            |(.., n)| Entry::Port(mem::Port {name: n.into(), dir: super::PortDir::Write})
        ),
        map(
            tuple((kw("readwriter"), arrow, spaced(identifier))),
            |(.., n)| Entry::Port(mem::Port {name: n.into(), dir: super::PortDir::ReadWrite})
        ),
        map(tuple((kw("read-latency"), arrow, spaced(decimal))), |(.., v)| Entry::ReadLatency(v)),
        map(tuple((kw("write-latency"), arrow, spaced(decimal))), |(.., v)| Entry::WriteLatency(v)),
        map(tuple((kw("read-under-write"), arrow, spaced(ruw))), |(.., v)| Entry::RUW(v)),
    ))(input)
}


fn arrow<'i>(input: &'i str) -> IResult<'i, ()> {
    use nom::Parser;

    spaced(kw("=>")).parse(input)
}

fn ruw<'i>(input: &'i str) -> IResult<'i, super::ReadUnderWrite> {
    use super::ReadUnderWrite as RUW;

    alt((
        value(RUW::Old, kw("old")),
        value(RUW::New, kw("new")),
        value(RUW::Undefined, kw("undefined")),
    ))(input)
}

