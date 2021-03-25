//! Parser utilities

/// Result type for our (sub)parsers
pub type IResult<'i, O> = nom::IResult<&'i str, O, Error<'i>>;


/// Error type for our (sub)parsers
pub type Error<'i> = nom::error::VerboseError<&'i str>;

