use enum_dispatch::*;

use std::borrow::Cow;
use std::io;
use std::path::PathBuf;

pub mod compress;
pub mod encrypt;
pub(crate) mod read;

pub use self::compress::*;
pub use self::encrypt::*;

#[derive(Debug, PartialEq)]
///In-memory data stream
///
///Represents an in-memory data stream
///It's the most common entry type
pub struct MemoryEntry<'a> {
    pub name: Cow<'a, str>,
    pub data: Cow<'a, [u8]>,
}

#[derive(Debug, PartialEq)]
///A file to be encoded as an entry
pub struct FileEntry {
    path: PathBuf,
}

#[enum_dispatch(Entry)]
#[derive(Debug, PartialEq)]
pub enum BaseEntry<'a> {
    Memory(MemoryEntry<'a>),
    File(FileEntry),
}

#[enum_dispatch]
pub trait Entry {
    fn name(&self) -> &str;
}

type EResult<T, E = Infallible> = Result<Option<T>, E>;

pub trait EntryExtract<'a>: Entry {
    type Extractor: io::Read;
    type Error: std::error::Error;

    fn extractor(&'a self) -> EResult<Self::Extractor, Self::Error>;
}

impl<'a> Entry for MemoryEntry<'a> {
    fn name(&self) -> &str {
        &self.name
    }
}

impl Entry for FileEntry {
    fn name(&self) -> &str {
        self.path.file_name().unwrap().to_str().unwrap()
    }
}

use std::convert::Infallible;
impl<'a> EntryExtract<'a> for MemoryEntry<'a> {
    type Extractor = &'a [u8];
    type Error = Infallible;

    fn extractor(&self) -> EResult<&[u8]> {
        Ok(Some(&self.data))
    }
}

impl<'a> EntryExtract<'a> for BaseEntry<'a> {
    type Extractor = &'a [u8];
    type Error = Infallible;

    fn extractor(&self) -> EResult<&[u8]> {
        match self {
            BaseEntry::Memory(e) => e.extractor(),
            _ => Ok(None),
        }
    }
}
