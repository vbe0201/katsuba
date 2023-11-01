use std::{mem, sync::Arc};

use byteorder::{ByteOrder, LE};
use katsuba_types::TypeList;
use once_cell::sync::Lazy;
use regex::bytes::Regex;

use super::{utils::bits_to_bytes, *};

const NO_FLAGS: u32 = SerializerFlags::empty().bits();
const ALL_FLAGS: u32 = SerializerFlags::all().bits();

#[inline]
fn read_u32(offset: usize, data: &[u8]) -> Option<u32> {
    data.get(offset..offset + mem::size_of::<u32>())
        .map(LE::read_u32)
}

#[inline]
fn maybe_serializer_flags(flags: Option<u32>) -> bool {
    // The position we check here is overlapping with:
    //
    // - **Object hashes:** - These are usually better distributed than flags
    //   and never this small. The exception is the null hash we account for.
    //
    // - **Zlib headers:** - When uncompressed, LSB 0 with 3 hash bytes. Also
    //   never this small. When compressed, LSB 1 and 3 length prefix bytes.
    //   Unless the length prefix is 0 (which is impossible), no ambiguity.
    let flags = flags.unwrap_or(ALL_FLAGS + 1);
    NO_FLAGS < flags && flags <= ALL_FLAGS
}

#[inline]
fn maybe_zlib_stream(offset: usize, data: &[u8]) -> bool {
    static HEADERS: [&[u8]; 4] = [b"\x78\x01", b"\x78\x9c", b"\x78\xda", b"\x78\x5e"];

    // Attempt to identify zlib streams cheaply by finding the magic header.
    data.get(offset..offset + 2)
        .map(|v| HEADERS.contains(&v))
        .unwrap_or(false)
}

#[inline]
fn check_bind_config(opts: &mut SerializerOptions, data: &[u8]) -> bool {
    // Checks the most common case: When data starts with the magic BINd bytes, it
    // is part of client files inside WAD archives. Fixed options are used for those.
    if data.get(0..4) == Some(BIND_MAGIC) {
        opts.flags = SerializerFlags::STATEFUL_FLAGS;
        opts.shallow = false;

        true
    } else {
        false
    }
}

#[inline]
fn set_compressed(opts: &mut SerializerOptions, data: &mut &[u8]) {
    opts.flags |= SerializerFlags::WITH_COMPRESSION;
    *data = &data[1..];
}

#[inline]
fn check_serialization_mode(opts: &mut SerializerOptions, offset: usize, data: &[u8]) {
    // A type hash is followed by the size of the remaining stream in bits
    // in deep mode. So we try to confirm this by trial and error.
    if let Some(maybe_bits) = read_u32(offset, data) {
        let maybe_bytes = bits_to_bytes(maybe_bits as _);
        opts.shallow = maybe_bytes != data.len();
    }
}

fn zlib_decompress(
    inflater: &mut Decompressor,
    out: &mut Vec<u8>,
    data: &[u8],
) -> Result<bool, Error> {
    match de::zlib_decompress(inflater, data, out) {
        Ok(()) => Ok(true),

        // Assume this was a false positive stream.
        Err(Error::Decompress(_)) => Ok(false),

        Err(e) => Err(e),
    }
}

fn check_length_prefix_types(opts: &mut SerializerOptions, data: &[u8]) {
    static ASCII_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"[ -~]{4,}").unwrap());

    for captures in ASCII_RE.captures_iter(data) {
        // There is always one match guaranteed in the captures.
        let mat = captures.get(0).unwrap();
        let sub = mat.as_bytes();

        // Try to determine the length prefix type.
        // Given the match, we always know these ranges exist.
        let big_len = read_u32(mat.start().saturating_sub(4), data).unwrap() as usize;
        let small_len = *data.get(mat.start().saturating_sub(1)).unwrap() as usize;

        // First, the obvious case: The 32-bit length prefix fits.
        if big_len == sub.len() {
            opts.flags &= !SerializerFlags::COMPACT_LENGTH_PREFIXES;
        }

        // Then, check if a compact small or large length prefix fits.
        let is_small = (small_len & 1 == 0b0) && (small_len >> 1) == sub.len();
        let is_large = (big_len & 1 == 0b1) && (big_len >> 1) == sub.len();
        if is_small || is_large {
            opts.flags |= SerializerFlags::COMPACT_LENGTH_PREFIXES;
        }
    }
}

pub struct Guesser {
    types: Arc<TypeList>,
    zlib: ZlibParts,
    opts: SerializerOptions,
}

impl Guesser {
    pub fn new(opts: SerializerOptions, types: Arc<TypeList>) -> Self {
        Self {
            types,
            zlib: ZlibParts::new(),
            opts,
        }
    }

    pub fn guess(mut self, data: &[u8]) -> Result<Serializer, Error> {
        // We perform only a baseline guess -- a pass that identifies and bases
        // off unambiguous properties of serialized data under the assumption
        // the stream is valid.
        self.baseline_guess(data)?;

        // What we don't know at this point:
        //
        // - Are enums compact or human-readable?
        // - What is the utilized property filter mask?

        Ok(Serializer {
            parts: SerializerParts {
                options: self.opts,
                types: self.types,
            },
            zlib_parts: self.zlib,
        })
    }

    fn baseline_guess<'a>(&'a mut self, mut data: &'a [u8]) -> Result<(), Error> {
        if check_bind_config(&mut self.opts, data) {
            return Ok(());
        }

        // First, check if we're dealing with a compressed object.
        if maybe_zlib_stream(4, data)
            && zlib_decompress(&mut self.zlib.inflater, &mut self.zlib.scratch1, data)?
        {
            self.opts.manual_compression = true;
            data = &self.zlib.scratch1;
        }

        // Here, we need to figure out the object layout.
        //
        // In the current position, we have one of the following:
        //
        // - Stateful serializer flags
        // - A chunk of the compression header
        // - An object hash (or null hash)
        //
        // See `maybe_serializer_flags` for a more detailed explanation
        // about data ambiguity and how we resolve it.
        let x = read_u32(0, data);

        if maybe_serializer_flags(x) {
            self.opts.flags = SerializerFlags::from_bits_truncate(x.unwrap());
            data = &data[4..];
        }

        if maybe_zlib_stream(5, data)
            && data.first() == Some(&1)
            && zlib_decompress(&mut self.zlib.inflater, &mut self.zlib.scratch2, &data[1..])?
        {
            self.opts.flags |= SerializerFlags::WITH_COMPRESSION;
            data = &self.zlib.scratch2;
        }

        // After decompression the state may have changed, so read again.
        let x = read_u32(0, data);
        let y = read_u32(1, data);

        // KingsIsle's implementation supports writing uncompressed data even when the
        // `WITH_COMPRESSION` bit is set. If we don't get a match for a given hash at
        // this position, then it is likely we stumbled across this behavior.
        let type_def = match (x, y) {
            // In this situation, `a` and `b` are candidates for type hashes. If `a`
            // is one, the stream is uncompressed. If `b` is one however, the stream
            // must be compressed.
            (Some(a), Some(b)) if a != 0 && b != 0 => {
                if let Some(type_def) = self.types.0.get(&a) {
                    Some(type_def)
                } else if let Some(type_def) = self.types.0.get(&b) {
                    // Here we expect `a`'s LSB to be the no compression marker.
                    (a & 0xFF == 0).then(|| {
                        set_compressed(&mut self.opts, &mut data);
                        type_def
                    })
                } else {
                    // Undefined type; we have to assume it is uncompressed.
                    None
                }
            }

            // Here we have a sequence of 5 null bytes, which means there is the
            // no compression marker and a null object hash. That still qualifies.
            (Some(0), Some(0)) => {
                set_compressed(&mut self.opts, &mut data);
                None
            }

            _ => None,
        };

        if type_def.is_some() {
            // First, try to guess the serialization mode.
            check_serialization_mode(&mut self.opts, 4, data);

            // Lastly, try to guess the type of length prefixes used if no
            // stateful serializer configuration was given.
            if !self.opts.flags.contains(SerializerFlags::STATEFUL_FLAGS) {
                check_length_prefix_types(&mut self.opts, data);
            }
        }

        Ok(())
    }
}
