use nom::{
    branch::alt, bytes::complete::*, combinator::*, error::context, multi::many1,
    number::complete::*, *,
};

use self::error::*;
use super::*;
use crate::entry::read::*;

use std::borrow::Cow;

pub mod error;

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
    move |i: &'a [u8]| f(slice_input(i, s)?).map_err(|e| Err::convert(e))
}

pub(crate) type Result<'a, O> = IResult<&'a [u8], O, ParserError<'a>>;

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
        let entries = entries.into_iter().map(|e| e.into()).collect();
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
        let (i, _) = context(
            "Invalid FARC mode",
            map_opt(be_u32, |m| if E::Mode == m { Some(true) } else { None }),
        )(i)?;
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
        let entries = entries.into_iter().map(|e| e.into()).collect();
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
        let (i, _) = context(
            "Invalid FARC mode",
            map_opt(be_u32, |m| if E::Mode == m { Some(true) } else { None }),
        )(i)?;
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
        use nom::{combinator::map, sequence::pair};
        let (_, entries) = slice(
            many1(map(pair(entry_read, be_u32), |(e, _)| e)),
            0x1C..0x1C + bs - 20,
        )(i0)?;
        let entries = entries.into_iter().map(|e| e.into()).collect();
        Ok((i, FutureArchive(BasicArchive { align, entries })))
    }
}

impl<'a> FutureArchives<'a> {
    pub fn read(i0: &'a [u8]) -> Result<'a, Self> {
        alt((
            map(FutureArchive::read, |a| FutureArchives::Base(a)),
            map(FutureArchive::read, |a| FutureArchives::Compress(a)),
            map(FutureArchive::read, |a| FutureArchives::Encrypt(a)),
            map(FutureArchive::read, |a| FutureArchives::CompressEncrypt(a)),
        ))(i0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::dbg_dmp;

    const INPUT: &[u8] = include_bytes!("../../assets/robmot_PV626.farc");
    const COMP: &[u8] = include_bytes!("../../assets/gm_module_tbl.farc");
    const FARC: &[u8] = include_bytes!("../../assets/pv_721_common.farc");
    const FUTURE: &[u8] = include_bytes!("../../assets/lenitm027.farc");

    #[test]
    fn read_base() {
        let (_, farc) = BaseArchive::read(INPUT).unwrap();
        let entry = BaseEntry::Memory(MemoryEntry {
            name: "mot_PV626.bin".into(),
            data: INPUT[0x22..][..15305208].into(),
        });
        assert_eq!(entry, farc.entries[0]);
    }
    #[test]
    fn read_compressed() {
        let (_, farc) = CompressArchive::read(COMP).unwrap();
        let entry: Compressor = CompressedEntry {
            entry: MemoryEntry {
                name: "gm_module_id.bin".into(),
                data: COMP[41..][..3827].into(),
            },
            original_len: 21050,
        }
        .into();
        assert_eq!(entry, farc.entries[0]);
    }
    #[test]
    fn read_extended_encrypt_compres() {
        let (_, farc) = ExtendArchive::<Encryptor<Compressor<'_>>>::read(FARC).unwrap();
        for entry in &farc.0.entries {
            println!("{}", &entry.name());
        }
        //pv_721_mouth.dsc
        //pv_721_scene.dsc
        //pv_721_success_mouth.dsc
        //pv_721_success_scene.dsc
        //pv_721_system.dsc
        assert_eq!(farc.0.entries[0].name(), "pv_721_mouth.dsc");
        assert_eq!(farc.0.entries[1].name(), "pv_721_scene.dsc");
        assert_eq!(farc.0.entries[2].name(), "pv_721_success_mouth.dsc");
        assert_eq!(farc.0.entries[3].name(), "pv_721_success_scene.dsc");
        assert_eq!(farc.0.entries[4].name(), "pv_721_system.dsc");
    }
    #[test]
    fn read_future_compressed() {
        let (_, farc) =
            dbg_dmp(FutureArchive::<CompressedEntry<'_>>::read, "future")(FUTURE).unwrap();
        for entry in &farc.0.entries {
            println!("{} {:#X}", entry.name(), entry.original_len);
        }
        assert_eq!(farc.0.entries[0].name(), "lenitm027_obj.bin");
        assert_eq!(farc.0.entries[1].name(), "lenitm027_tex.bin");
    }
}
