use super::*;

pub(crate) fn string(i: &[u8]) -> Result<Cow<str>> {
    is_not::<_, _, ParserError<'_>>("\x00")(i)
        .map(|(i2, s)| (&i2[1..], String::from_utf8_lossy(s)))
        .map_err(|_| Err::Error(ParserError(i, ParserErrorKind::StringOverflow)))
}
use nom::error::ParseError;
pub(crate) fn be_usize<'a, E: ParseError<&'a [u8]>>(i: &'a [u8]) -> IResult<&'a [u8], usize, E> {
    map(be_u32, |x| x as usize)(i)
}
use std::slice::SliceIndex;
pub(crate) fn slice_input<S>(i: &[u8], s: S) -> core::result::Result<&[u8], Err<ParserError<'_>>>
where
    S: SliceIndex<[u8], Output = [u8]>,
{
    i.get(s)
        .ok_or(Err::Failure(ParserError(i, ParserErrorKind::InvalidOffset)))
}
pub(crate) fn slice<'a, S, F, O, E>(f: F, s: S) -> impl FnOnce(&'a [u8]) -> Result<O>
where
    S: SliceIndex<[u8], Output = [u8]>,
    F: Fn(&'a [u8]) -> IResult<&'a [u8], O, E>,
    ParserError<'a>: From<E>,
{
    move |i: &'a [u8]| f(slice_input(i, s)?).map_err(Err::convert)
}

pub(crate) type Result<'a, O> = IResult<&'a [u8], O, ParserError<'a>>;
