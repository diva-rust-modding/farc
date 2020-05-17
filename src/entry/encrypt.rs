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
impl<'a> Encrypt for Compressor<'a> {
    type Decrypt = CompressedEntry<'a>;
}

const KEY: &[u8] = b"project_diva.bin";
use aes_frast::*;
use std::io::Cursor;
impl<'a> EntryExtract<'a> for Encryptor<BaseEntry<'a>> {
    type Extractor = Cursor<Vec<u8>>;
    type Error = std::io::Error;

    fn extractor(&'a self) -> EResult<Self::Extractor, Self::Error> {
        match self {
            //get rid of this unwrap
            Self::Encrypted(e) => {
                println!("encrypted");
                let extractor = match e.extractor().unwrap() {
                    Some(a) => a,
                    None => return Ok(None),
                };
                println!("extracting");
                let mut keys = vec![0; 44];
                aes_core::setkey_dec_auto(&KEY, &mut keys);
                let mut plain = vec![0; extractor.len()];
                aes_with_operation_mode::ecb_dec(extractor, &mut plain, &keys);
                Ok(Some(Cursor::new(plain)))
            }
            _ => Ok(None),
        }
    }
}

use flate2::read::GzDecoder;
impl<'a> EntryExtract<'a> for Encryptor<Compressor<'a>> {
    type Extractor = GzDecoder<Cursor<Vec<u8>>>;
    type Error = std::io::Error;

    fn extractor(&'a self) -> EResult<Self::Extractor, Self::Error> {
        match self {
            //get rid of this unwrap
            Self::Encrypted(e) => {
                println!("{:X?} len={:#X?}", &e.entry.data[..4], e.entry.data.len());
                let mut keys = vec![0; 44];
                aes_core::setkey_dec_auto(&KEY, &mut keys);
                let extractor = &e.entry.data[..];
                let mut plain = vec![0; extractor.len()];
                aes_with_operation_mode::ecb_dec(extractor, &mut plain, &keys);
                let decomp = GzDecoder::new(Cursor::new(plain));
                println!("resturn");
                Ok(Some(decomp))
            }
            _ => Ok(None),
        }
    }
}
