use super::*;

#[derive(Debug, PartialEq)]
pub enum Encryptor<E: Encrypt> {
    Encrypt(E),
    Encrypted(E::Decrypt),
}

pub trait Encrypt: Entry {
    type Decrypt: Entry;

    fn encrypt(self) -> Encryptor<Self>
    where
        Self: Sized,
    {
        Encryptor::Encrypt(self)
    }
}

impl<E: Encrypt> Entry for Encryptor<E> {
    fn name(&self) -> &str {
        match self {
            Encryptor::Encrypt(e) => e.name(),
            Encryptor::Encrypted(e) => e.name(),
        }
    }
}

impl<'a> Encrypt for MemoryEntry<'a> {
    type Decrypt = Self;
}
impl<'a> Encrypt for BaseEntry<'a> {
    type Decrypt = MemoryEntry<'a>;
}
impl<'a> Encrypt for CompressEntry<'a> {
    type Decrypt = CompressedEntry<'a>;
}
