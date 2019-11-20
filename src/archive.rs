use crate::entry::compress::*;
use crate::entry::*;

pub enum  GenericArchive<'a> {
    Base(BaseArchive<'a>),
    Compress(CompressArchive<'a>),
    Extended(ExtendedArchives<'a>),
    Future(FutureArchives<'a>)
}

pub type BaseArchive<'a> = BasicArchive<BaseEntry<'a>>;
pub type CompressArchive<'a> = BasicArchive<Compressor<'a>>;

#[derive(Debug, PartialEq)]
pub struct BasicArchive<E> {
    pub align: u32,
    pub entries: Vec<E>,
}

#[derive(Debug, PartialEq)]
pub struct ExtendArchive<E: Entry>(pub BasicArchive<E>);

#[derive(Debug, PartialEq)]
pub enum ExtendedArchives<'a> {
    Base(ExtendArchive<BaseEntry<'a>>),
    Compress(ExtendArchive<Compressor<'a>>),
    Encrypt(ExtendArchive<Encryptor<BaseEntry<'a>>>),
    CompressEncrypt(ExtendArchive<Encryptor<Compressor<'a>>>),
}

#[derive(Debug, PartialEq)]
pub struct FutureArchive<E: Entry>(pub BasicArchive<E>);

#[derive(Debug, PartialEq)]
pub enum FutureArchives<'a> {
    Base(FutureArchive<BaseEntry<'a>>),
    Compress(FutureArchive<Compressor<'a>>),
    Encrypt(FutureArchive<Encryptor<BaseEntry<'a>>>),
    CompressEncrypt(FutureArchive<Encryptor<Compressor<'a>>>),
}
