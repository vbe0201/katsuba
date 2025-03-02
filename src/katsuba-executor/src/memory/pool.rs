use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

use crossbeam_queue::ArrayQueue;

/// A pool which stores byte vectors and hands them out on demand.
///
/// Memory will be reused to avoid allocations when buffers are
/// available for use.
#[derive(Debug)]
pub struct Pool {
    queue: ArrayQueue<Vec<u8>>,
}

impl Pool {
    /// Creates a new pool with an upper bound of byte vectors
    /// it can store at the same time.
    pub fn new(capacity: usize) -> Arc<Self> {
        Arc::new(Self {
            queue: ArrayQueue::new(capacity),
        })
    }

    /// Inserts a new byte vector into the pool, which is initialized
    /// by the given closure.
    pub fn create_with<F>(&self, f: F)
    where
        F: FnOnce(&mut Vec<u8>),
    {
        let mut buf = Vec::new();
        f(&mut buf);

        // We just care about preserving the memory that may have
        // been allocated by `f`, not the contents it inserted.
        buf.clear();
        let _ = self.queue.push(buf);
    }

    /// Gets an element from the pool or creates a new one to
    /// be inserted when the ref is dropped.
    ///
    /// New vectors not part of the pool yet start empty and
    /// will not trigger memory allocation.
    pub fn get(self: Arc<Self>) -> PoolRef {
        let inner = self.queue.pop().unwrap_or_default();

        PoolRef {
            pool: self,
            inner: Some(inner),
        }
    }
}

/// A reference to a byte vector from a [`Pool`], enabling mutable
/// and immutable access to the element.
///
/// When this value is dropped, the element will be inserted back
/// into the pool.
#[derive(Debug)]
pub struct PoolRef {
    pool: Arc<Pool>,
    inner: Option<Vec<u8>>,
}

impl Deref for PoolRef {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        self.inner.as_ref().unwrap()
    }
}

impl DerefMut for PoolRef {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.as_mut().unwrap()
    }
}

impl Drop for PoolRef {
    fn drop(&mut self) {
        let mut value = self.inner.take().unwrap();
        value.clear();

        let _ = self.pool.queue.push(value);
    }
}
