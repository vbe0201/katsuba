use crc32fast::Hasher;

/// Computes the CRC checksum over `data` using KI's
/// algorithm.
pub fn hash(data: &[u8]) -> u32 {
    let mut hasher = Hasher::new_with_initial(u32::MAX);
    hasher.update(data);
    hasher.finalize() ^ u32::MAX
}
