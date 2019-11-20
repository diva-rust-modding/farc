use nom::{bytes::complete::*, combinator::*, multi::many1, number::complete::*, branch::alt, error::context, *};

use super::*;
use crate::entry::read::*;

fn be_usize(i: &[u8]) -> IResult<&[u8], usize> {
    map(be_u32, |x| x as usize)(i)
}

impl GenericArchive<'_> {
    pub fn read(i0: &[u8]) -> IResult<&[u8], GenericArchive> {
        alt((
            map(BaseArchive::read, GenericArchive::Base),
            map(CompressArchive::read, GenericArchive::Compress),
            map(ExtendedArchives::read, GenericArchive::Extended),
            map(FutureArchives::read, GenericArchive::Future),
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
        let (i, _) = context("Invalid FARC mode", map_opt(be_u32, |m| if E::Mode == m { Some(true) } else { None }))(i)?;
        //skip 4 bytes
        let i = &i[4..];
        let (i, align) = be_u32(i)?;
        let (i, _) = context("Future FARC detected", map_opt(be_u32, |m| if 0 == m { Some(true) } else { None }))(i)?;
        //panic!("{} {} {}", bs, align, bs-0xC);
        let entry_read = |i: &'a [u8]| E::read(i0, i);
        let (_, entries) = many1(entry_read)(&i0[0x1C..][..bs-20])?;
        let entries = entries.into_iter().map(|e| e.into()).collect();
        Ok((i, ExtendArchive(BasicArchive { align, entries })))
    }
}
impl<'a> ExtendedArchives<'a> {
    pub fn read(i0: &'a [u8]) -> IResult<&'a [u8], Self> {
        alt((
            map(ExtendArchive::read, ExtendedArchives::Base),
            map(ExtendArchive::read, ExtendedArchives::Compress),
            map(ExtendArchive::read, ExtendedArchives::Encrypt),
            map(ExtendArchive::read, ExtendedArchives::CompressEncrypt),
        ))(i0)
    }
}
impl<'a, E: ExtendEntry<'a>> FutureArchive<E> {
    fn read(i0: &'a [u8]) -> IResult<&'a [u8], Self> {
        let (i, _) = tag("FARC")(i0)?;
        let (i, bs) = be_usize(i)?;
        let (i, _) = context("Invalid FARC mode", map_opt(be_u32, |m| if E::Mode == m { Some(true) } else { None }))(i)?;
        //skip 4 bytes
        let i = &i[4..];
        let (i, align) = be_u32(i)?;
        let (i, _) = context("Normal FARC detected", map_opt(be_u32, |m| if 1 == m { Some(true) } else { None }))(i)?;
        //skip 8 bytes
        let i = &i[8..];
        let entry_read = |i: &'a [u8]| E::read(i0, i);
        use nom::{sequence::pair, combinator::map};
        let (_, entries) = many1(map(pair(entry_read, be_u32), |(e, _)| e))(&i0[0x20..][..bs-24])?;
        let entries = entries.into_iter().map(|e| e.into()).collect();
        Ok((i, FutureArchive(BasicArchive { align, entries })))
    }
}

impl<'a> FutureArchives<'a> {
    pub fn read(i0: &'a [u8]) -> IResult<&'a [u8], Self> {
        alt((
            map(FutureArchive::read, |a|  FutureArchives::Base(a) ),
            map(FutureArchive::read, |a|  FutureArchives::Compress(a) ),
            map(FutureArchive::read, |a|  FutureArchives::Encrypt(a) ),
            map(FutureArchive::read, |a|  FutureArchives::CompressEncrypt(a) ),
        ))(i0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::dbg_dmp;

    const INPUT: &[u8] = include_bytes!("../assets/robmot_PV626.farc");
    const COMP: &[u8] = include_bytes!("../assets/gm_module_tbl.farc");
    const FARC: &[u8] = include_bytes!("../assets/pv_721_common.farc");
    const FUTURE: &[u8] = include_bytes!("../assets/lenitm027.farc");

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
        let (_, farc) = dbg_dmp(FutureArchive::<CompressedEntry<'_>>::read, "future")(FUTURE).unwrap();
        for entry in &farc.0.entries {
            println!("{} {:#X}", entry.name(), entry.original_len);
        }
        assert_eq!(farc.0.entries[0].name(), "lenitm027_obj.bin");
        assert_eq!(farc.0.entries[1].name(), "lenitm027_tex.bin");
    }
}
