use std::{
    borrow::Cow,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use sharded_slab::pool::{OwnedRef, OwnedRefMut, Pool};

/// An in-memory buffer for I/O tasks on the executor.
#[derive(Debug)]
pub enum Buffer<'a> {
    /// A handle to memory allocated on the current thread.
    Current(Cow<'a, [u8]>),
    /// A handle to memory from the pool.
    Threaded(PoolRef),
}

impl Buffer<'_> {
    /// Downgrades the given buffer into immutable state.
    ///
    /// No data mutation should be attempted after a call to this
    /// because it will likely lead to panic.
    pub fn downgrade(self) -> Self {
        match self {
            Self::Threaded(PoolRef::Mut(orm, pool)) => {
                Self::Threaded(PoolRef::Owned(orm.downgrade(), pool))
            }

            buf => buf,
        }
    }
}

impl Buffer<'static> {
    /// Gets an owned buffer as a mutable vector reference for
    /// I/O operations.
    ///
    /// # Panics
    ///
    /// Panics when the buffer is borrowed.
    pub fn as_vec(&mut self) -> &mut Vec<u8> {
        match self {
            Self::Current(buf) => match buf {
                Cow::Owned(buf) => buf,
                Cow::Borrowed(..) => unimplemented!(),
            },

            Self::Threaded(pr) => pr,
        }
    }
}

/// An owned reference to pooled memory between multiple worker threads.
#[derive(Debug)]
pub enum PoolRef {
    Mut(OwnedRefMut<Vec<u8>>, Arc<Pool<Vec<u8>>>),
    Owned(OwnedRef<Vec<u8>>, Arc<Pool<Vec<u8>>>),
}

impl PoolRef {
    fn clear(&self) {
        match self {
            Self::Mut(orm, pool) => pool.clear(orm.key()),
            Self::Owned(or, pool) => pool.clear(or.key()),
        };
    }
}

impl Deref for PoolRef {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Mut(orm, _) => orm,
            Self::Owned(or, _) => or,
        }
    }
}

impl DerefMut for PoolRef {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::Mut(orm, _) => orm,
            Self::Owned(..) => unimplemented!(),
        }
    }
}

impl<'a> Buffer<'a> {
    /// Extends the lifetime of the buffer to `'static` so that it
    /// can be moved into the executor.
    ///
    /// # Safety
    ///
    /// The implementation must ensure that borrowed buffers do not
    /// outlive their source, e.g. by making sure all tasks complete
    /// timely on a join call.
    #[inline]
    pub unsafe fn extend_lifetime(self) -> Buffer<'static> {
        // SAFETY: Similar to docs example for `transmute`, caller
        // takes responsibility in case of a borrowed buffer.
        std::mem::transmute::<Self, Buffer<'static>>(self)
    }

    /// Creates a file buffer from a static slice.
    #[inline]
    pub const fn current_borrowed(buf: &'a [u8]) -> Self {
        Self::Current(Cow::Borrowed(buf))
    }

    /// Creates a file buffer from an owned byte vector.
    #[inline]
    pub fn current_owned(buf: Vec<u8>) -> Self {
        Self::Current(Cow::Owned(buf))
    }

    /// Creates a buffer from an existing [`PoolRef`].
    #[inline]
    pub fn pooled(pr: PoolRef) -> Self {
        Self::Threaded(pr)
    }

    /// Clears the buffer.
    ///
    /// For pooled memory, this recycles the memory back into the pool
    /// as soon as no other thread is holding onto it anymore.
    ///
    /// For everything else, this is a no-op.
    #[inline]
    pub fn clear(&self) {
        if let Self::Threaded(pool_ref) = self {
            pool_ref.clear();
        }
    }
}

impl Deref for Buffer<'_> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Current(buf) => buf,
            Self::Threaded(pool_ref) => pool_ref,
        }
    }
}

impl DerefMut for Buffer<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::Threaded(pr) => pr,
            Self::Current(buf) => match buf {
                Cow::Owned(buf) => buf,
                Cow::Borrowed(..) => unimplemented!(),
            },
        }
    }
}
