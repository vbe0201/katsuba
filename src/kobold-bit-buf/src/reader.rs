use std::{io, marker::PhantomData, ptr, slice};

// The maximum number of bits that can be stored in lookahead.
//
// We target to have an amount between 56 and 63 bits in the
// buffer. Since we only refill by whole bytes, it means the
// low 3 bits never change.
const BUFFER_SIZE: u32 = u64::BITS - 1;

// The maximum number of bits to be consumed after a refill.
//
// Since we refill by whole bytes only, this is the smallest
// value where a whole byte doesn't fit in anymore.
const CONSUMABLE_BITS: u32 = BUFFER_SIZE & !7;

#[inline(always)]
unsafe fn read_64_le(ptr: *const u8) -> u64 {
    // SAFETY: Caller must provide a valid pointer.
    let value = unsafe { ptr::read_unaligned(ptr.cast::<u64>()) };
    value.to_le()
}

// There are no stable branch prediction hints for us.
// We emulate them by nudging the compiler into optimizing
// around the paths which do not call this function.
#[cold] // :)
#[inline(always)]
fn cold() {}

/// A buffer which enables bit-based deserialization of data.
///
/// Individual bit reading starts at the LSB of the byte, working
/// towards the MSB.
///
/// When reading bits, users must manually call [`Self::refill_bits`]
/// first and do appropriate checks based on how many bits are left.
///
/// When wanting to read from full byte boundaries with some stale
/// buffered bits, [`Self::invalidate_and_realign_ptr`] can help.
#[derive(Debug)]
pub struct BitReader<'a> {
    // Pointer to the next byte where the bit lookahead
    // buffer will be fetched from.
    ptr: *const u8,

    // A marker pointer past which it is not safe anymore
    // to take the fast refill path which blindly copies.
    safeguard: *const u8,

    // One-past-the-end pointer in the spanned byte view.
    end: *const u8,

    // The pre-fetched lookahead bufer to extract bits at.
    lookahead: u64,

    // The number of bits available for consumption from
    // the `lookahead` buffer.
    remaining: u32,

    // Marker to teach borrowchk this is holding onto a slice.
    _m: PhantomData<&'a [u8]>,
}

impl<'a> BitReader<'a> {
    /// Creates a new [`BitReader`] over a given byte slice.
    pub const fn new(data: &'a [u8]) -> Self {
        let (ptr, len) = (data.as_ptr(), data.len());

        // SAFETY: All pointer arithmetic in bounds or one past the end.
        unsafe {
            Self {
                ptr,
                safeguard: ptr.add(len.saturating_sub(7)),
                end: ptr.add(len),
                lookahead: 0,
                remaining: 0,
                _m: PhantomData,
            }
        }
    }

    #[inline(always)]
    fn can_read_in_fast_path(&self) -> bool {
        self.ptr < self.safeguard
    }

    // Gets the remaining untouched bytes in the reader.
    #[inline]
    pub fn untouched_bytes(&self) -> usize {
        // SAFETY: Byte pointers are derived from the same object,
        // with `ptr <= end` being an internally maintained invariant.
        unsafe { self.end.offset_from(self.ptr) as usize }
    }

    /// Gets the total number of remaining bits in the reader.
    #[inline]
    pub fn remaining_bits(&self) -> usize {
        (self.untouched_bytes() << 3) + self.remaining as usize
    }

    /// Gets the bits currently buffered in the reader.
    #[inline]
    pub fn buffered_bits(&self) -> u32 {
        self.remaining
    }

    /// Invalidates the current bit lookahead and resets the pointer
    /// back to the first untouched byte.
    ///
    /// Untouched in this case means no partial bit reads overlapping
    /// with the memory region of a byte have happened yet.
    #[inline(always)]
    pub fn realign_to_byte(&mut self) {
        // SAFETY: Decrementing the pointer is fine since we move within
        // a fraction of the increment done by a refill operation.
        self.ptr = unsafe { self.ptr.sub(self.remaining as usize >> 3) };

        self.lookahead = 0;
        self.remaining = 0;
    }

    // SAFETY: Caller must make sure `self.ptr` is valid for 8 byte read.
    #[inline]
    unsafe fn refill_branchless(&mut self) {
        // Read from current bit pointer and prefill the entire lookahead.
        self.lookahead |= unsafe { read_64_le(self.ptr) } << self.remaining;

        // Advance the read cursor for the next refill.
        //
        // This code seemingly increases the number of instructions
        // from 3 (sub, shr, add) to 4 (add, shr, and, sub), but with
        // dedicated bitfield extraction support it stays at 3.
        //
        // Note that the dependency chain decreases from 3 to 2 however,
        // which may result in higher throughput.
        unsafe {
            self.ptr = self.ptr.add(CONSUMABLE_BITS as usize >> 3);
            self.ptr = self.ptr.sub((self.remaining as usize >> 3) & 7);
        }

        // Update bit count to reflect full buffer.
        self.remaining |= CONSUMABLE_BITS;
    }

    #[inline]
    fn refill_slow(&mut self) {
        while self.remaining < CONSUMABLE_BITS {
            if self.ptr == self.end {
                cold();
                break;
            }

            // SAFETY: `ptr` hasn't reached the buffer end yet.
            unsafe {
                self.lookahead |= (*self.ptr as u64) << self.remaining;
                self.ptr = self.ptr.add(1);
            }

            self.remaining += u8::BITS;
        }
    }

    /// Refills the bit lookahead buffer and returns the number of
    /// available bits for consumption.
    ///
    /// When this buffer is exhausted, another refill must be done.
    pub fn refill_bits(&mut self) -> u32 {
        debug_assert!(self.ptr <= self.end);
        debug_assert!(self.remaining <= BUFFER_SIZE);

        if self.can_read_in_fast_path() {
            // SAFETY: We have enough bytes for an unchecked refill.
            unsafe {
                self.refill_branchless();
            }
        } else {
            cold();
            self.refill_slow();
        }

        self.remaining
    }

    /// Returns the next `count` bits from the internal buffer without removing
    /// them, if available.
    #[inline]
    pub fn peek(&mut self, count: u32) -> io::Result<u64> {
        if count <= CONSUMABLE_BITS && count <= self.remaining {
            Ok(self.lookahead & ((1 << count) - 1))
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "attempted to read out of bounds",
            ))
        }
    }

    /// Removes `count` bits from the internal buffer, if available.
    #[inline]
    pub fn consume(&mut self, count: u32) -> io::Result<()> {
        if count <= self.remaining {
            self.lookahead >>= count;
            self.remaining -= count;

            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "attempted to read out of bounds",
            ))
        }
    }

    /// Reads `nbytes` raw bytes from the internal buffer.
    ///
    /// These are borrowed from the underlying byte view without
    /// copying them.
    pub fn read_bytes(&mut self, count: usize) -> io::Result<&'a [u8]> {
        if count <= self.untouched_bytes() {
            // SAFETY: A bounds check was done and an appropriate lifetime is
            // inferred through the function signature.
            unsafe {
                let value = slice::from_raw_parts(self.ptr, count);
                self.ptr = self.ptr.add(count);
                Ok(value)
            }
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "attempted to read out of bounds",
            ))
        }
    }
}

// SAFETY: The `BitReader` API does not expose the internal pointers
// in ways which can compromise Rust's safety guarantees.
unsafe impl Send for BitReader<'_> {}
unsafe impl Sync for BitReader<'_> {}
