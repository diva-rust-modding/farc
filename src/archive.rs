use crate::entry::compress::*;
use crate::entry::*;

pub enum GenericArchive<'a> {
    Base(BaseArchive<'a>),
    Compress(CompressArchive<'a>),
    Extended(ExtendedArchives<'a>),
    Future(FutureArchives<'a>),
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

use std::ops::Deref;
impl<E> Deref for BasicArchive<E> {
    type Target = [E];

    fn deref(&self) -> &Self::Target {
        &self.entries
    }
}

impl<E: Entry> Deref for ExtendArchive<E> {
    type Target = BasicArchive<E>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<E: Entry> Deref for FutureArchive<E> {
    type Target = BasicArchive<E>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ExtendedArchives<'_> {
    pub fn len(&self) -> usize {
        match self {
            Self::Base(a) => a.len(),
            Self::Compress(a) => a.len(),
            Self::Encrypt(a) => a.len(),
            Self::CompressEncrypt(a) => a.len(),
        }
    }
}

impl FutureArchives<'_> {
    pub fn len(&self) -> usize {
        match self {
            Self::Base(a) => a.len(),
            Self::Compress(a) => a.len(),
            Self::Encrypt(a) => a.len(),
            Self::CompressEncrypt(a) => a.len(),
        }
    }
}

impl GenericArchive<'_> {
    pub fn magic(&self) -> &'static str {
        match self {
            Self::Base(_) => "FArc",
            Self::Compress(_) => "FArC",
            _ => "FARC",
        }
    }
    pub fn len(&self) -> usize {
        match self {
            Self::Base(a) => a.len(),
            Self::Compress(a) => a.len(),
            Self::Extended(a) => a.len(),
            Self::Future(a) => a.len(),
        }
    }
}

impl<E> IntoIterator for BasicArchive<E> {
    type Item = E;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.into_iter()
    }
}

impl<E: Entry> IntoIterator for ExtendArchive<E> {
    type Item = E;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<E: Entry> IntoIterator for FutureArchive<E> {
    type Item = E;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
