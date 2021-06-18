//! Parser utilities

#[cfg(test)]
mod tests;


use nom::Parser;
use nom::bytes::complete::{tag, take_while};
use nom::character::complete::{satisfy, space0};
use nom::combinator::{not, peek, value};
use nom::error::context;
use nom::sequence::{preceded, tuple};

/// Result type for our (sub)parsers
pub type IResult<'i, O> = nom::IResult<&'i str, O, Error<'i>>;


/// Error type for our (sub)parsers
pub type Error<'i> = nom::error::VerboseError<&'i str>;


/// Parse an identifier
///
/// The parser will consume the longest sequence of alphanumeric characters and
/// '_'. However, the parser will return an error if the first character is a
/// numeric character.
pub fn identifier(input: &str) -> IResult<&str> {
    context(
        "expected identifier",
        preceded(peek(not(satisfy(char::is_numeric))), take_while(is_identifier_char))
    )(input)
}


/// Parse a decimal numeral
pub fn decimal<O>(input: &str) -> IResult<O>
    where O: std::str::FromStr
{
    use nom::combinator::{recognize, success};
    use nom::branch::alt;

    let sign = alt((value((), tag("+")), value((), tag("-")), success(())));

    context(
        "expected decimal numeral",
        nom::combinator::map_res(
            recognize(tuple((sign, take_while(char::is_numeric)))),
            str::parse
        )
    )(input)
}


/// Parse a comma, skipping preceding whitespace
pub fn comma(input: &str) -> IResult<()> {
    spaced(op(",")).parse(input)
}


/// Parse a left parantheses, skipping preceding whitespace
pub fn lp(input: &str) -> IResult<()> {
    spaced(op("(")).parse(input)
}


/// Parse a right parantheses, skipping preceding whitespace
pub fn rp(input: &str) -> IResult<()> {
    spaced(op(")")).parse(input)
}


/// Create a parser for the specified keyword
///
/// Contrary to operators, keywords are a subset of identifiers in the sense
/// that a parser for identifiers would also accept a keyword. I.e. they consist
/// of characters which could appear in an identifier. Hence, they need to be
/// separated from identifiers by whitespace.
pub fn kw<'i>(keyword: &'static str) -> impl nom::Parser<&'i str, (), Error<'i>> {
    value((), tuple((tag(keyword), peek(not(satisfy(is_identifier_char))))))
}


/// Create a parser for the specified operator
///
/// Operators are strings which do not contain characters which could appear in
/// an identifier.
pub fn op<'i>(operator: &'static str) -> impl nom::Parser<&'i str, (), Error<'i>> {
    value((), tag(operator))
}


/// Parse line endings, skipping preceding whitespace
///
/// This parser consumes line endings, optionally preceded by whitespace. If no
/// line ending is recognized, this parser will yield an error.
pub fn le<'i>(input: &'i str) -> IResult<'i, ()> {
    nom::multi::fold_many1(spaced(nom::character::complete::line_ending), (), |_, _| ())(input)
}


/// Create a parser which discards any space before applying another parser
///
/// This function wraps the given parser in another parser which will be
/// returned to the caller. The returned parser will consume any spaces and
/// tabs, then apply the wrapped parser.
pub fn spaced<'i, O>(
    inner: impl nom::Parser<&'i str, O, Error<'i>>
) -> impl nom::Parser<&'i str, O, Error<'i>> {
    preceded(space0, inner)
}


/// Check whether the character is allowed in identifiers
fn is_identifier_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

