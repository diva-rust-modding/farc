use crate::entry::compress::*;
use crate::entry::*;

pub enum  GenericArchive<'a> {
    Base(BaseArchive<'a>),
    Compress(CompressArchive<'a>),
}

#[derive(Debug, PartialEq)]
pub struct BaseArchive<'a> {
    pub align: u32,
    pub entries: Vec<BaseEntry<'a>>,
}

#[derive(Debug, PartialEq)]
pub struct CompressArchive<'a> {
    pub align: u32,
    pub entries: Vec<CompressEntry<'a>>,
}
