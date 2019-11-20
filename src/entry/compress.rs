use flate2::read::GzDecoder;
use flate2::write::GzEncoder;

use enum_dispatch::*;

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

use std::io::Read;
#[enum_dispatch(Read)]
pub enum Decompressor<'a> {
    Decompress(GzDecoder<&'a [u8]>),
    Uncompressed(&'a [u8]),
}

impl<'a> Read for Decompressor<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            Decompressor::Decompress(gz) => gz.read(buf),
            Decompressor::Uncompressed(b) => b.read(buf),
        }
    }
}

impl<'a> EntryExtract<'a> for CompressedEntry<'a> {
    type Extractor = GzDecoder<&'a [u8]>;

    fn extractor(&'a self) -> Self::Extractor {
        GzDecoder::new(&self.entry.data)
    }
}

impl<'a> EntryExtract<'a> for Compressor<'a> {
    type Extractor = Decompressor<'a>;

    fn extractor(&'a self) -> Self::Extractor {
        match self {
            Compressor::Compress(e) => Decompressor::Uncompressed(e.extractor()),
            Compressor::Compressed(e) => Decompressor::Decompress(e.extractor()),
        }
    }
}

impl<'a> Entry for CompressedEntry<'a> {
    fn name(&self) -> &str {
        &self.entry.name
    }
}
