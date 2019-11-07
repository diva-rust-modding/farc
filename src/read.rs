use nom::{bytes::complete::*, combinator::*, number::complete::*, multi::many1, *};

use super::*;
use crate::entry::read::*;

fn be_usize(i: &[u8]) -> IResult<&[u8], usize> {
    map(be_u32, |x| x as usize)(i)
}

impl<'a> BaseArchive<'a> {
    pub fn read(i0: &'a [u8]) -> IResult<&'a [u8], Self> {
        let (i, _) = tag("FArc")(i0)?;
        let (i, bs) = be_usize(i)?;
        let (i, align) = be_usize(i)?;
        //panic!("{} {} {}", bs, align, bs-0xC);
        let (_, descriptors) = many1(MemoryEntryDescriptor::read)(&i0[0xC..][..bs])?;
        let entries = descriptors.into_iter().map(|d| d.into_entry(i0)).collect();
        Ok((i, BaseArchive(entries)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &[u8] = include_bytes!("../assets/robmot_PV626.farc");

    #[test]
    fn read_base() {
        let (_, farc) = BaseArchive::read(INPUT).unwrap();
        let entry = MemoryEntry { name: "mot_PV626.bin".into(), data: INPUT[0x22..][..15305208].into() };
        assert_eq!(entry, farc.0[0]);
    }
}
