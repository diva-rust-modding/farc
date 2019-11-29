use nom::error::{ErrorKind, ParseError};
use thiserror::*;

#[derive(Error, Debug)]
#[error("A parser error has occured: {1}")]
pub struct ParserError<'a>(pub &'a [u8], pub ParserErrorKind<'a>);

#[derive(Error, Debug)]
pub enum ParserErrorKind<'a> {
    #[error("Invalid slice offset, most likely corrupt archive")]
    InvalidOffset,
    #[error("Invalid magic expected farc, found {0:?}")]
    InvalidMagic(&'a str),
    #[error("Invalid mode, expected {expected} found {found}.\nMost likely corrupted archive or encrypted future tone archive")]
    InvalidMode { expected: u32, found: u32 },
    #[error("Invalid version detected, expected {expected} found {found}.\nMost likely found future tone archive while expecting extended and vice versa")]
    InvalidVersion { expected: u32, found: u32 },
    #[error("String overflew, couldn't find null byte")]
    StringOverflow,
    #[error("{0:?}")]
    Other(ErrorKind),
}

impl<'a> ParseError<&'a [u8]> for ParserError<'a> {
    fn from_error_kind(input: &'a [u8], kind: ErrorKind) -> Self {
        Self(input, ParserErrorKind::Other(kind))
    }

    fn append(_: &[u8], _: ErrorKind, other: Self) -> Self {
        other
    }
}

impl<'a> From<(&'a [u8], ErrorKind)> for ParserError<'a> {
    fn from((i, err): (&'a [u8], ErrorKind)) -> Self {
        Self::from_error_kind(i, err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn convert() {
        let i: &[u8] = &[];
        let err = (i, ErrorKind::IsNot);
        let _err2: ParserError = err.into();
    }
}
