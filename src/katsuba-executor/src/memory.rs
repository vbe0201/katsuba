use std::{
    borrow::Cow,
    mem,
    ops::{Deref, DerefMut},
};

mod pool;
pub(crate) use pool::{Pool, PoolRef};

#[derive(Debug)]
enum BufferInner<'a> {
    Pooled(PoolRef),
    Cow(Cow<'a, [u8]>),
}

/// An in-memory buffer for I/O tasks on the executor.
///
/// Buffers come in different flavors, they can be created from
/// byte slices and vectors as well as over pooled memory.
#[derive(Debug)]
pub struct Buffer<'a>(BufferInner<'a>);

impl<'a> Buffer<'a> {
    /// Extends the lifetime of the buffer to `'static` so that it
    /// can be moved into the executor.
    ///
    /// # Safety
    ///
    /// The implementation must ensure that borrowed buffers do not
    /// outlive the owner, e.g. by making sure all tasks complete
    /// timely on a join call.
    #[inline]
    pub unsafe fn extend_lifetime(self) -> Buffer<'static> {
        // SAFETY: Similar to docs example, caller takes
        // responsibility in case of a borrowed buffer.
        unsafe { mem::transmute(self) }
    }

    /// Creates a file buffer from a borrowed slice.
    #[inline]
    pub const fn borrowed(buf: &'a [u8]) -> Self {
        Self(BufferInner::Cow(Cow::Borrowed(buf)))
    }

    /// Creates a file buffer from an owned byte vector.
    #[inline]
    pub const fn owned(buf: Vec<u8>) -> Self {
        Self(BufferInner::Cow(Cow::Owned(buf)))
    }

    /// Creates a buffer from an existing [`PoolRef`].
    #[inline]
    pub(crate) fn pooled(pr: PoolRef) -> Self {
        Self(BufferInner::Pooled(pr))
    }
}

impl Deref for Buffer<'_> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        match &self.0 {
            BufferInner::Pooled(pr) => pr,
            BufferInner::Cow(buf) => buf,
        }
    }
}

impl DerefMut for Buffer<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match &mut self.0 {
            BufferInner::Pooled(pr) => pr,
            BufferInner::Cow(buf) => match buf {
                Cow::Owned(buf) => buf,
                Cow::Borrowed(..) => unimplemented!(),
            },
        }
    }
}
