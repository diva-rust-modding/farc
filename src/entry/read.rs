use nom::{bytes::complete::*, combinator::*, number::complete::*, *};
use std::borrow::Cow;

use super::*;

fn string<'a>(i: &'a [u8]) -> IResult<&'a [u8], Cow<'a, str>> {
    is_not("\x00")(i).map(|(i2, s)| (&i2[1..], String::from_utf8_lossy(s)))
}
fn be_usize(i: &[u8]) -> IResult<&[u8], usize> {
    map(be_u32, |x| x as usize)(i)
}

pub trait Descriptor<'a> {
    type Entry: Entry;

    fn read(i: &'a [u8]) -> IResult<&'a [u8], Self>
    where
        Self: Sized;
    fn into_entry(self, i: &'a [u8]) -> Self::Entry;
}

#[derive(Debug)]
pub struct MemoryEntryDescriptor<'a> {
    pub name: Cow<'a, str>,
    pub pos: usize,
    pub len: usize,
}

#[derive(Debug)]
pub struct CompressedEntryDescriptor<'a> {
    pub name: Cow<'a, str>,
    pub pos: usize,
    pub len: usize,
    pub original_len: u32,
}

impl<'a> Descriptor<'a> for MemoryEntryDescriptor<'a> {
    type Entry = MemoryEntry<'a>;

    fn read(i: &'a [u8]) -> IResult<&'a [u8], Self> {
        let (i, name) = string(i)?;
        let (i, pos) = be_usize(i)?;
        let (i, len) = be_usize(i)?;
        Ok((i, MemoryEntryDescriptor { name, pos, len }))
    }
    fn into_entry(self, i: &'a [u8]) -> Self::Entry {
        MemoryEntry {
            name: self.name,
            data: i[self.pos..][..self.len].into(),
        }
    }
}

impl<'a> Descriptor<'a> for CompressedEntryDescriptor<'a> {
    type Entry = CompressedEntry<'a>;

    fn read(i: &'a [u8]) -> IResult<&'a [u8], Self> {
        let (i, name) = string(i)?;
        let (i, pos) = be_usize(i)?;
        let (i, len) = be_usize(i)?;
        let (i, original_len) = be_u32(i)?;
        Ok((
            i,
            CompressedEntryDescriptor {
                name,
                pos,
                len,
                original_len,
            },
        ))
    }
    fn into_entry(self, i: &'a [u8]) -> Self::Entry {
        CompressedEntry {
            entry: MemoryEntry {
                name: self.name,
                data: i[self.pos..][..self.len].into(),
            },
            original_len: self.original_len,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &[u8] = include_bytes!("../../assets/robmot_PV626.farc");
    const COMP: &[u8] = include_bytes!("../../assets/gm_module_tbl.farc");

    #[test]
    fn read_memory() {
        let (_, desc) = MemoryEntryDescriptor::read(&INPUT[0xC..]).unwrap();
        assert_eq!(desc.name, "mot_PV626.bin");
        assert_eq!(desc.pos, 34);
        assert_eq!(desc.len, 15305208);
        let entry = desc.into_entry(INPUT);
        assert_eq!(&entry.data[..4], &[0x20, 0, 0, 0]);
    }

    #[test]
    fn read_compressed() {
        let (_, desc) = CompressedEntryDescriptor::read(&COMP[0xC..]).unwrap();
        assert_eq!(desc.name, "gm_module_id.bin");
        assert_eq!(desc.pos, 41);
        assert_eq!(desc.len, 3827);
        assert_eq!(desc.original_len, 21050);
        let entry = desc.into_entry(COMP);
        assert_eq!(&entry.entry.data[..4], &[0x1F, 0x8B, 8, 8]);
    }
}
