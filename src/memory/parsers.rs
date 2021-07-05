//! Parsers related to memory elements

use nom::branch::alt;
use nom::combinator::{iterator, map, value};
use nom::sequence::tuple;

use crate::indentation::Indentation;
use crate::parsers::{self, IResult, decimal, identifier, kw, le, op, spaced};
use crate::types::Type;
use crate::types::parsers::r#type;
use crate::info::parse as info;


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
    let mut depth: Option<super::Depth> = Default::default();
    let mut read_latency: Option<super::Latency> = Default::default();
    let mut write_latency: Option<super::Latency> = Default::default();
    let mut ports: Vec<super::Port> = Default::default();
    let mut ruw: super::ReadUnderWrite = Default::default();
    (&mut entries).for_each(|e| match e {
        Entry::DataType(t)      => data_type = Some(t),
        Entry::Depth(v)         => depth = Some(v),
        Entry::Port(p)          => ports.push(p),
        Entry::ReadLatency(v)   => read_latency = Some(v),
        Entry::WriteLatency(v)  => write_latency = Some(v),
        Entry::RUW(v)           => ruw = v,
    });

    let mut res = super::Memory::new(
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


enum Entry {
    DataType(Type),
    Depth(super::Depth),
    Port(super::Port),
    ReadLatency(super::Latency),
    WriteLatency(super::Latency),
    RUW(super::ReadUnderWrite),
}


fn entry<'i>(input: &'i str) -> IResult<'i, Entry> {
    alt((
        map(tuple((kw("data-type"), arrow, spaced(r#type))), |(.., t)| Entry::DataType(t)),
        map(tuple((kw("depth"), arrow, spaced(decimal))), |(.., v)| Entry::Depth(v)),
        map(
            tuple((kw("reader"), arrow, spaced(identifier))),
            |(.., n)| Entry::Port(super::Port {name: n.into(), kind: super::PortKind::Read})
        ),
        map(
            tuple((kw("writer"), arrow, spaced(identifier))),
            |(.., n)| Entry::Port(super::Port {name: n.into(), kind: super::PortKind::Write})
        ),
        map(
            tuple((kw("readwriter"), arrow, spaced(identifier))),
            |(.., n)| Entry::Port(super::Port {name: n.into(), kind: super::PortKind::ReadWrite})
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

