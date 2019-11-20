use flate2::read::GzDecoder;
use flate2::write::GzEncoder;

use super::*;

#[derive(Debug, PartialEq)]
pub struct CompressedEntry<'a> {
    pub(crate) entry: MemoryEntry<'a>,
    pub(crate) original_len: u32,
}

#[enum_dispatch(Entry)]
#[derive(Debug, PartialEq)]
pub enum Compressor<'a> {
    Compress(BaseEntry<'a>),
    Compressed(CompressedEntry<'a>),
}

pub enum Decompressor<'a> {
    Decompress(GzDecoder<&'a [u8]>),
    Uncompressed(&'a [u8]),
}

impl<'a> EntryExtract<'a> for CompressedEntry<'a> {
    type Extractor = GzDecoder<&'a [u8]>;

    fn extractor(&'a self) -> Self::Extractor {
        GzDecoder::new(&self.entry.data)
    }
}

impl<'a> Entry for CompressedEntry<'a> {
    fn name(&self) -> &str {
        &self.entry.name
    }
}
