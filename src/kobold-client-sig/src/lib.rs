//! A crate for working with client signature files.
//!
//! Client signatures are produced by invoking the game client with the `-CS`
//! CLI flag, resulting in a `ClientSig.bin` file. Those are used to affirm
//! client authenticity to the server.
//!
//! The files are typically obtained in encrypted state, so this crate provides
//! the necessary decryption routines.

#![deny(rust_2018_idioms, rustdoc::broken_intra_doc_links)]
#![forbid(unsafe_code)]

use std::io;

use base64::{prelude::BASE64_STANDARD, Engine};
use byteorder::{ReadBytesExt, LE};
use kobold_utils::thiserror::{self, Error};
use rsa::{
    pkcs1::DecodeRsaPrivateKey,
    pkcs1v15::SigningKey,
    signature::{RandomizedSigner, SignatureEncoding},
    Oaep, RsaPrivateKey,
};
use sha1::Sha1;

const SECRET: &[u8] = b"\xE1\x6C\xD6\xAE\x52\x90\x49\xF1\xF1\xBB\xE9\xEB\xB3\xA6\xDB\x3C";

/// Errors that may occur when working with ClientSig data.
#[derive(Debug, Error)]
pub enum Error {
    /// An attempt to parse supplied ClientSig data failed.
    #[error("{0}")]
    Io(#[from] io::Error),

    /// Failed to decrypt the ClientSig chunks.
    #[error("{0}")]
    Rsa(#[from] rsa::Error),
}

/// A private key representation which implements cryptographic operations in
/// relation to client signatures.
#[derive(Clone)]
pub struct PrivateKey(RsaPrivateKey);

impl PrivateKey {
    /// Constructs a new private key for cryptographic operations.
    ///
    /// `key` is a PKCS#1-encoded RSA private key in PEM format.
    pub fn new(key: &str) -> rsa::pkcs1::Result<Self> {
        RsaPrivateKey::from_pkcs1_pem(key).map(Self)
    }

    /// Prepares an access key for invoking the `-CS` CLI flag in order to dump
    /// a client signature file.
    ///
    /// The resulting string must be passed as the argument to the flag.
    pub fn make_access_key(self) -> String {
        let signing_key = SigningKey::<Sha1>::new(self.0);

        let signature = signing_key.sign_with_rng(&mut rand::thread_rng(), SECRET);
        BASE64_STANDARD.encode(signature.to_bytes())
    }

    /// Decrypts the contents of an encrypted `ClientSig.bin` file and returns
    /// a byte vector containing the plaintext data.
    pub fn decrypt_sig(&self, mut data: &[u8]) -> Result<Vec<u8>, Error> {
        let mut out = Vec::with_capacity(data.len());

        while !data.is_empty() {
            // Read the chunk size and make sure we have the data.
            let chunk_size = data.read_u32::<LE>()? as usize;
            if data.len() < chunk_size {
                return Err(Error::Io(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "not enough chunks in ClientSig",
                )));
            }

            // Decrypt the next chunk and append it to the output buffer.
            let (chunk, remainder) = data.split_at(chunk_size);
            let mut decrypted_chunk = self.0.decrypt(Oaep::new::<Sha1>(), chunk)?;

            out.append(&mut decrypted_chunk);
            data = remainder;
        }

        Ok(out)
    }
}
