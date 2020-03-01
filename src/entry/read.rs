use nom::number::complete::*;

use super::*;
use crate::read::utilities::*;

#[enum_dispatch]
pub trait ReadEntry<'a>: Entry + Sized {
    fn read(i0: &'a [u8], i: &'a [u8]) -> Result<'a, Self>;
}

pub trait BasicEntry<'a>: ReadEntry<'a> {}
pub trait ExtendEntry<'a>: ReadEntry<'a> {
    const MODE: u32;
}

impl<'a> ReadEntry<'a> for MemoryEntry<'a> {
    fn read(i0: &'a [u8], i: &'a [u8]) -> Result<'a, Self> {
        let (i, name) = string(i)?;
        let (i, pos) = be_usize(i)?;
        let (i, len) = be_usize(i)?;
        let data = slice_input(i0, pos..pos + len)?.into();
        Ok((i, MemoryEntry { name, data }))
    }
}

//Should be obselete once `enum_dispatch` supports multiple traits
//See: https://gitlab.com/antonok/enum_dispatch/issues/3
impl<'a> ReadEntry<'a> for BaseEntry<'a> {
    fn read(i0: &'a [u8], i: &'a [u8]) -> Result<'a, Self> {
        let (i, entry) = MemoryEntry::read(i0, &i)?;
        Ok((i, entry.into()))
    }
}

impl<'a> ReadEntry<'a> for CompressedEntry<'a> {
    fn read(i0: &'a [u8], i: &'a [u8]) -> Result<'a, Self> {
        let (i, entry) = MemoryEntry::read(i0, &i)?;
        let (i, original_len) = be_u32(i)?;
        Ok((
            i,
            CompressedEntry {
                entry,
                original_len,
            },
        ))
    }
}

//Should be obselete once `enum_dispatch` supports multiple traits
//See: https://gitlab.com/antonok/enum_dispatch/issues/3
impl<'a> ReadEntry<'a> for Compressor<'a> {
    fn read(i0: &'a [u8], i: &'a [u8]) -> Result<'a, Self> {
        let (i, entry) = CompressedEntry::read(i0, &i)?;
        Ok((i, entry.into()))
    }
}

impl<'a, E> ReadEntry<'a> for Encryptor<E>
where
    E: Encrypt,
    E::Decrypt: ReadEntry<'a>,
{
    fn read(i0: &'a [u8], i: &'a [u8]) -> Result<'a, Self> {
        let (i, entry) = E::Decrypt::read(i0, &i)?;
        Ok((i, Encryptor::Encrypted(entry)))
    }
}

impl<'a> BasicEntry<'a> for MemoryEntry<'a> {}
impl<'a> BasicEntry<'a> for BaseEntry<'a> {}
impl<'a> BasicEntry<'a> for CompressedEntry<'a> {}
impl<'a> BasicEntry<'a> for Compressor<'a> {}

impl<'a> ExtendEntry<'a> for MemoryEntry<'a> {
    const MODE: u32 = 0;
}
impl<'a> ExtendEntry<'a> for BaseEntry<'a> {
    const MODE: u32 = 0;
}
impl<'a> ExtendEntry<'a> for CompressedEntry<'a> {
    const MODE: u32 = 2;
}
impl<'a> ExtendEntry<'a> for Compressor<'a> {
    const MODE: u32 = 2;
}
impl<'a, E> ExtendEntry<'a> for Encryptor<E>
where
    E: ExtendEntry<'a> + Encrypt,
    E::Decrypt: ReadEntry<'a>,
{
    const MODE: u32 = E::MODE | 4;
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
