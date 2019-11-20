use nom::{bytes::complete::*, combinator::*, multi::many1, number::complete::*, branch::alt, *};

use super::*;
use crate::entry::read::*;

fn be_usize(i: &[u8]) -> IResult<&[u8], usize> {
    map(be_u32, |x| x as usize)(i)
}

impl GenericArchive<'_> {
    pub fn read(i0: &[u8]) -> IResult<&[u8], GenericArchive> {
        alt((
            map(BaseArchive::read, |a| GenericArchive::Base(a)),
            map(CompressArchive::read, |a| GenericArchive::Compress(a)),
        ))(i0)
    }
}

impl<'a, E: BasicEntry<'a>> BasicArchive<E> {
    fn read_magic(i0: &'a [u8], magic: &'static str) -> IResult<&'a [u8], Self> {
        let (i, _) = tag(magic)(i0)?;
        let (i, bs) = be_usize(i)?;
        let (i, align) = be_u32(i)?;
        //panic!("{} {} {}", bs, align, bs-0xC);
        let entry_read = |i: &'a [u8]| E::read(i0, i);
        let (_, entries) = many1(entry_read)(&i0[0xC..][..bs-4])?;
        let entries = entries.into_iter().map(|e| e.into()).collect();
        Ok((i, BasicArchive { align, entries }))
    }
}
impl<'a> BaseArchive<'a> {
    pub fn read(i0: &'a [u8]) -> IResult<&'a [u8], Self> {
        Self::read_magic(i0, "FArc")
    }
}
impl<'a> CompressArchive<'a> {
    pub fn read(i0: &'a [u8]) -> IResult<&'a [u8], Self> {
        Self::read_magic(i0, "FArC")
    }
}
impl<'a, E: ExtendEntry<'a>> ExtendArchive<E> {
    pub fn read(i0: &'a [u8]) -> IResult<&'a [u8], Self> {
        let (i, _) = tag("FARC")(i0)?;
        let (i, bs) = be_usize(i)?;
        let (i, mode) = be_usize(i)?;
        //skip 4 bytes
        let i = &i[4..];
        let (i, align) = be_u32(i)?;
        //panic!("{} {} {}", bs, align, bs-0xC);
        let entry_read = |i: &'a [u8]| E::read(i0, i);
        let (_, entries) = many1(entry_read)(&i0[0xC..][..bs-20])?;
        let entries = entries.into_iter().map(|e| e.into()).collect();
        Ok((i, ExtendArchive(BasicArchive { align, entries })))
    }
}
impl<'a> ExtendedArchives<'a> {
    pub fn read(i0: &'a [u8]) -> IResult<&'a [u8], Self> {
        alt((
            map(ExtendArchive::read, |a| ExtendedArchives::Base(a)),
            map(ExtendArchive::read, |a| ExtendedArchives::Compress(a)),
            map(ExtendArchive::read, |a| ExtendedArchives::Encrypt(a)),
            map(ExtendArchive::read, |a| ExtendedArchives::CompressEncrypt(a)),
        ))(i0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &[u8] = include_bytes!("../assets/robmot_PV626.farc");
    const COMP: &[u8] = include_bytes!("../assets/gm_module_tbl.farc");
    const FARC: &[u8] = include_bytes!("../assets/pv_721_common.farc");

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
        let entry: CompressEntry = CompressedEntry {
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
    fn read_extended() {
        let (_, farc) = ExtendedArchives::read(FARC).unwrap();
    }
}
