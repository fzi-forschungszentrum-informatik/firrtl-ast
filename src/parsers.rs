//! Parser utilities

use nom::bytes::streaming::take_while;
use nom::character::streaming::{satisfy, space0};
use nom::combinator::{not, peek};
use nom::error::context;
use nom::sequence::tuple;

/// Result type for our (sub)parsers
pub type IResult<'i, O> = nom::IResult<&'i str, O, Error<'i>>;


/// Error type for our (sub)parsers
pub type Error<'i> = nom::error::VerboseError<&'i str>;


/// Parse an identifier
///
/// The parser will consume the longest sequence of alphanumeric characters and
/// '_'. However, the parser will return an error if the first character is a
/// numeric character.
///
/// The returned parser will consume any spaces and tabs preceding the identifier.
pub fn identifier(input: &str) -> IResult<&str> {
    context(
        "expected identifier",
        nom::combinator::map(
            tuple((space0, peek(not(satisfy(char::is_numeric))), take_while(is_identifier_char))),
            |(_, _, s)| s
        )
    )(input)
}


/// Check whether the character is allowed in identifiers
fn is_identifier_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

