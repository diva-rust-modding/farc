pub mod entry;
pub mod read;

use crate::entry::*;

#[derive(Debug, PartialEq)]
pub struct BaseArchive<'a>(Vec<MemoryEntry<'a>>);
