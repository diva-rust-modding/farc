use nom::{bytes::complete::*, combinator::*, multi::many1, number::complete::*, *};

use super::*;
use crate::entry::read::*;

fn be_usize(i: &[u8]) -> IResult<&[u8], usize> {
    map(be_u32, |x| x as usize)(i)
}

impl<'a> BaseArchive<'a> {
    pub fn read(i0: &'a [u8]) -> IResult<&'a [u8], Self> {
        let (i, _) = tag("FArc")(i0)?;
        let (i, bs) = be_usize(i)?;
        let (i, align) = be_u32(i)?;
        //panic!("{} {} {}", bs, align, bs-0xC);
        let (_, descriptors) = many1(MemoryEntryDescriptor::read)(&i0[0xC..][..bs-4])?;
        let entries = descriptors
            .into_iter()
            .map(|d| d.into_entry(i0).into())
            .collect();
        Ok((i, BaseArchive { align, entries }))
    }
}

impl<'a> CompressArchive<'a> {
    pub fn read(i0: &'a [u8]) -> IResult<&'a [u8], Self> {
        let (i, _) = tag("FArC")(i0)?;
        let (i, bs) = be_usize(i)?;
        let (i, align) = be_u32(i)?;
        //panic!("{} {} {}", bs, align, bs-0xC);
        let (_, descriptors) = many1(CompressedEntryDescriptor::read)(&i0[0xC..][..bs-4])?;
        let entries = descriptors
            .into_iter()
            .map(|d| d.into_entry(i0).into())
            .collect();
        Ok((i, CompressArchive { align, entries }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &[u8] = include_bytes!("../assets/robmot_PV626.farc");
    const COMP: &[u8] = include_bytes!("../assets/gm_module_tbl.farc");

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
}
