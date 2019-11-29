use nom::{
    branch::alt, bytes::complete::*, combinator::*, error::context, multi::many1,
    number::complete::*, sequence::pair, *,
};

use self::error::*;
use super::*;
use crate::entry::read::*;

use std::borrow::Cow;

pub mod error;
#[cfg(test)]
mod tests;
pub(crate) mod utilities;

use self::utilities::*;

impl<'a> GenericArchive<'a> {
    pub fn read(i0: &'a [u8]) -> Result<'a, Self> {
        alt((
            map(BaseArchive::read, GenericArchive::Base),
            map(CompressArchive::read, GenericArchive::Compress),
            map(ExtendedArchives::read, GenericArchive::Extended),
            map(FutureArchives::read, GenericArchive::Future),
        ))(i0)
    }
}

impl<'a, E: BasicEntry<'a>> BasicArchive<E> {
    fn read_magic(i0: &'a [u8], magic: &'static str) -> Result<'a, Self> {
        let (i, _) = tag(magic)(i0)?;
        let (i, bs) = be_usize(i)?;
        let (i, align) = be_u32(i)?;
        //panic!("{} {} {}", bs, align, bs-0xC);
        let entry_read = |i: &'a [u8]| E::read(i0, i);
        let (_, entries) = slice(many1(entry_read), 0xC..0xC + bs - 4)(i0)?;
        Ok((i, BasicArchive { align, entries }))
    }
}
impl<'a> BaseArchive<'a> {
    pub fn read(i0: &'a [u8]) -> Result<'a, Self> {
        Self::read_magic(i0, "FArc")
    }
}
impl<'a> CompressArchive<'a> {
    pub fn read(i0: &'a [u8]) -> Result<'a, Self> {
        Self::read_magic(i0, "FArC")
    }
}
impl<'a, E: ExtendEntry<'a>> ExtendArchive<E> {
    pub fn read(i0: &'a [u8]) -> Result<'a, Self> {
        let (i, _) = tag("FARC")(i0)?;
        let (i, bs) = be_usize(i)?;
        let (i, mode) = be_u32(i)?;
        if mode != E::MODE {
            return Err(Err::Error(ParserError(
                i,
                ParserErrorKind::InvalidMode {
                    expected: E::MODE,
                    found: mode,
                },
            )));
        }
        //skip 4 bytes
        let i = &i[4..];
        let (i, align) = be_u32(i)?;
        let (i, _) = context(
            "Future FARC detected",
            map_opt(be_u32, |m| if 0 == m { Some(true) } else { None }),
        )(i)?;
        //panic!("{} {} {}", bs, align, bs-0xC);
        let entry_read = |i: &'a [u8]| E::read(i0, i);
        let (_, entries) = slice(many1(entry_read), 0x1C..0x1C + bs - 20)(i0)?;
        Ok((i, ExtendArchive(BasicArchive { align, entries })))
    }
}
impl<'a> ExtendedArchives<'a> {
    pub fn read(i0: &'a [u8]) -> Result<'a, Self> {
        alt((
            map(ExtendArchive::read, ExtendedArchives::Base),
            map(ExtendArchive::read, ExtendedArchives::Compress),
            map(ExtendArchive::read, ExtendedArchives::Encrypt),
            map(ExtendArchive::read, ExtendedArchives::CompressEncrypt),
        ))(i0)
    }
}
impl<'a, E: ExtendEntry<'a>> FutureArchive<E> {
    pub fn read(i0: &'a [u8]) -> Result<'a, Self> {
        let (i, _) = tag("FARC")(i0)?;
        let (i, bs) = be_usize(i)?;
        let (i, mode) = be_u32(i)?;
        if mode != E::MODE {
            return Err(Err::Error(ParserError(
                i,
                ParserErrorKind::InvalidMode {
                    expected: E::MODE,
                    found: mode,
                },
            )));
        }
        //skip 4 bytes
        let i = &i[4..];
        let (i, align) = be_u32(i)?;
        let (i, _) = context(
            "Normal FARC detected",
            map_opt(be_u32, |m| if 1 == m { Some(true) } else { None }),
        )(i)?;
        //skip 8 bytes
        let i = &i[8..];
        let entry_read = |i: &'a [u8]| E::read(i0, i);
        let (_, entries) = slice(
            many1(map(pair(entry_read, be_u32), |(e, _)| e)),
            0x20..0x20 + bs - 24,
        )(i0)?;
        Ok((i, FutureArchive(BasicArchive { align, entries })))
    }
}

impl<'a> FutureArchives<'a> {
    pub fn read(i0: &'a [u8]) -> Result<'a, Self> {
        alt((
            map(FutureArchive::read, FutureArchives::Base),
            map(FutureArchive::read, FutureArchives::Compress),
            map(FutureArchive::read, FutureArchives::Encrypt),
            map(FutureArchive::read, FutureArchives::CompressEncrypt),
        ))(i0)
    }
}
