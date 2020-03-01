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
use aesstream::*;
use crypto::aes::*;
use crypto::aessafe::*;
use crypto::blockmodes::*;
impl<'a> EntryExtract<'a> for Encryptor<BaseEntry<'a>> {
    type Extractor = EcbReader<AesSafe128Decryptor, NoPadding, &'a [u8]>;
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
                let decryptor =
                    EcbReader::new_ecb(extractor, AesSafe128Decryptor::new(KEY), NoPadding);
                Ok(Some(decryptor))
            }
            _ => Ok(None),
        }
    }
}

use flate2::read::GzDecoder;
impl<'a> EntryExtract<'a> for Encryptor<Compressor<'a>> {
    type Extractor = EcbReader<AesSafe128Decryptor, NoPadding, &'a [u8]>;
    type Error = std::io::Error;

    fn extractor(&'a self) -> EResult<Self::Extractor, Self::Error> {
        match self {
            //get rid of this unwrap
            Self::Encrypted(e) => {
                println!("{:X?} len={:#X?}", &e.entry.data[..4], e.entry.data.len());
                let decrypt =
                    EcbReader::new_ecb(&e.entry.data[..], AesSafe128Decryptor::new(KEY), NoPadding);
                //let decomp = GzDecoder::new(decrypt);
                println!("resturn");
                Ok(Some(decrypt))
            }
            _ => Ok(None),
        }
    }
}
