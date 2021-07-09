//! Parser utilities

#[cfg(test)]
mod tests;


use nom::Parser;
use nom::bytes::complete::{tag, take_while};
use nom::character::complete::{char as chr, satisfy, space0};
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


/// Parse an unquoted string
///
/// This function parses the inner of a string literal or info attribute. It
/// parses unescaped characters if they are not in `special` and characters
/// escaped with a backslash. `\n` and `\t` are special in this regard as these
/// are parsed as newline and tab characters respectively.
pub fn unquoted_string<'i>(input: &'i str, special: &[char]) -> IResult<'i, String> {
    use nom::combinator::{iterator, verify};
    use nom::branch::alt;
    use nom::character::complete::anychar;

    let mut chars = iterator(
        input,
        alt((
            value('\n', tag("\\n")),
            value('\t', tag("\\t")),
            preceded(chr('\\'), anychar),
            verify(anychar, |c| *c != '\\' && !special.contains(c)),
        ))
    );
    let res = (&mut chars).collect();
    chars.finish().map(|(i, _)| (i, res))
}


/// Parse a decimal numeral
pub fn decimal<O>(input: &str) -> IResult<O>
    where O: std::str::FromStr
{
    use nom::combinator::{map_res, recognize};

    context(
        "expected decimal numeral",
        map_res(
            recognize(tuple((sign, take_while(char::is_numeric)))),
            str::parse
        )
    )(input)
}


/// Parse a floating point numeral
pub fn float<O: std::str::FromStr>(input: &str) -> IResult<O> {
    use nom::branch::alt;
    use nom::combinator::{map_res, recognize};

    let format = tuple((
        sign,
        take_while(char::is_numeric),
        chr('.'),
        take_while(char::is_numeric),
        alt((
            peek(not(chr('E'))),
            value((), tuple((chr('E'), sign, take_while(char::is_numeric))))
        )),
    ));

    context("expected floating point numeral", map_res(recognize(format), str::parse))(input)
}


/// Parse an optional plus or minus sign
fn sign(input: &str) -> IResult<()> {
    use nom::{branch::alt, combinator::success};

    alt((value((), tag("+")), value((), tag("-")), success(())))(input)
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
pub fn is_identifier_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '$'
}

