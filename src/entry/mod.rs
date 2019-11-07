use enum_dispatch::*;

use std::borrow::Cow;
use std::io;

pub(crate) mod read;
pub mod compress;
pub mod encrypt;

use self::compress::*;
use self::encrypt::*;

#[derive(Debug, PartialEq)]
///In-memory data stream
///
///Represents an in-memory data stream
///It's the most common entry type
pub struct MemoryEntry<'a> {
    pub name: Cow<'a, str>,
    pub data: Cow<'a, [u8]>,
}

#[enum_dispatch]
pub trait Entry {
    fn name(&self) -> &str;
}

pub trait EntryExtract<'a>: Entry {
    type Extractor: io::Read;

    fn extractor(&'a self) -> Self::Extractor;
}

impl<'a> Entry for MemoryEntry<'a> {
    fn name(&self) -> &str {
        &self.name
    }
}
