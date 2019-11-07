use flate2::read::GzDecoder;
use flate2::write::GzEncoder;

use super::*;

#[derive(Debug, PartialEq)]
pub struct CompressedEntry<'a>(pub(crate) MemoryEntry<'a>, pub(crate)  u32);

#[enum_dispatch(Entry)]
#[derive(Debug, PartialEq)]
pub enum CompressEntry<'a> {
    Compress(MemoryEntry<'a>),
    Compressed(CompressedEntry<'a>),
}

pub enum Decompressor<'a> {
    Decompress(GzDecoder<&'a [u8]>),
    Uncompressed(&'a [u8]),
}

impl<'a> EntryExtract<'a> for CompressedEntry<'a> {
    type Extractor = GzDecoder<&'a [u8]>;

    fn extractor(&'a self) -> Self::Extractor {
        GzDecoder::new(&self.0.data)
    }
}

impl<'a> Entry for CompressedEntry<'a> {
    fn name(&self) -> &str {
        &self.0.name
    }
}
