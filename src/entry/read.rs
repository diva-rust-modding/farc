use nom::{bytes::complete::*, combinator::*, number::complete::*, error::context, *};
use std::borrow::Cow;

use super::*;

fn string(i: &[u8]) -> IResult<&[u8], Cow<str>> {
    context("string: failed to read ASCII string", is_not("\x00"))(i).map(|(i2, s)| (&i2[1..], String::from_utf8_lossy(s)))
}
fn be_usize(i: &[u8]) -> IResult<&[u8], usize> {
    map(be_u32, |x| x as usize)(i)
}

#[enum_dispatch]
pub trait ReadEntry<'a> : Entry + Sized {
    fn read(i0: &'a [u8], i:&'a [u8]) -> IResult<&'a [u8], Self>;
}

pub trait BasicEntry<'a>: ReadEntry<'a> {}
pub trait ExtendEntry<'a>: ReadEntry<'a> {
    const Mode: u32;
}

impl<'a> ReadEntry<'a> for MemoryEntry<'a> {
    fn read(i0: &'a [u8], i:&'a [u8]) -> IResult<&'a [u8], Self> {
        let (i, name) = string(i)?;
        let (i, pos) = be_usize(i)?;
        let (i, len) = be_usize(i)?;
        let data = i0[pos..][..len].into();
        Ok((i, MemoryEntry{ name, data }))
    }
}

//Should be obselete once `enum_dispatch` supports multiple traits
//See: https://gitlab.com/antonok/enum_dispatch/issues/3
impl<'a> ReadEntry<'a> for BaseEntry<'a> {
    fn read(i0: &'a [u8], i:&'a [u8]) -> IResult<&'a [u8], Self> {
        let (i, entry) = MemoryEntry::read(i0, &i)?;
        Ok((i, entry.into()))
    }
}

impl<'a> ReadEntry<'a> for CompressedEntry<'a> {
    fn read(i0: &'a [u8], i:&'a [u8]) -> IResult<&'a [u8], Self> {
        let (i, entry) = MemoryEntry::read(i0, &i)?;
        let (i, original_len) = be_u32(i)?;
        Ok((i, CompressedEntry { entry, original_len }))
    }
}

//Should be obselete once `enum_dispatch` supports multiple traits
//See: https://gitlab.com/antonok/enum_dispatch/issues/3
impl<'a> ReadEntry<'a> for CompressEntry<'a> {
    fn read(i0: &'a [u8], i:&'a [u8]) -> IResult<&'a [u8], Self> {
        let (i, entry) = CompressedEntry::read(i0, &i)?;
        Ok((i, entry.into()))
    }
}

impl<'a, E: ReadEntry<'a> + Encrypt> ReadEntry<'a> for Encryptor<E> {
    fn read(i0: &'a [u8], i:&'a [u8]) -> IResult<&'a [u8], Self> {
        let (i, entry) = E::read(i0, &i)?;
        Ok((i, entry.encrypt()))
    }
}

impl<'a> BasicEntry<'a> for MemoryEntry<'a> {}
impl<'a> BasicEntry<'a> for BaseEntry<'a> {}
impl<'a> BasicEntry<'a> for CompressedEntry<'a> {}
impl<'a> BasicEntry<'a> for CompressEntry<'a> {}

impl<'a> ExtendEntry<'a> for MemoryEntry<'a> {
    const Mode: u32 = 0;
}
impl<'a> ExtendEntry<'a> for BaseEntry<'a> {
    const Mode: u32 = 0;
}
impl<'a> ExtendEntry<'a> for CompressedEntry<'a> {
    const Mode: u32 = 2;
}
impl<'a> ExtendEntry<'a> for CompressEntry<'a> {
    const Mode: u32 = 2;
}
impl<'a, E: ExtendEntry<'a> + Encrypt> ExtendEntry<'a> for Encryptor<E> {
    const Mode: u32 = E::Mode | 4;
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &[u8] = include_bytes!("../../assets/robmot_PV626.farc");
    const COMP: &[u8] = include_bytes!("../../assets/gm_module_tbl.farc");

    #[test]
    fn read_memory() {
        let (_, entry) = MemoryEntry::read(INPUT, &INPUT[0xC..]).unwrap();
        assert_eq!(entry.name, "mot_PV626.bin");
        assert_eq!(&entry.data[..4], &[0x20, 0, 0, 0]);
    }

    #[test]
    fn read_compressed() {
        let (_, comp) = CompressedEntry::read(COMP, &COMP[0xC..]).unwrap();
        assert_eq!(&comp.entry.name, "gm_module_id.bin");
        assert_eq!(comp.original_len, 21050);
        assert_eq!(&comp.entry.data[..4], &[0x1F, 0x8B, 8, 8]);
    }
}
