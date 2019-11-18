use crate::entry::compress::*;
use crate::entry::*;

pub enum  GenericArchive<'a> {
    Base(BaseArchive<'a>),
    Compress(CompressArchive<'a>),
}

pub type BaseArchive<'a> = BasicArchive<BaseEntry<'a>>;
pub type CompressArchive<'a> = BasicArchive<CompressEntry<'a>>;

#[derive(Debug, PartialEq)]
pub struct BasicArchive<E> {
    pub align: u32,
    pub entries: Vec<E>,
}
