use std::{io, option::IntoIter as OptionIter, sync::Arc};

use super::Task;
use crate::memory::{Pool, PoolRef};

/// An executor flavor which carries out every task on the
/// current thread in sequential order.
pub struct Current {
    pool: Arc<Pool>,
}

impl Current {
    #[inline]
    pub(super) fn new() -> Self {
        // Since we are on a single thread, we only need one element
        // in the pool that we will always reuse. The synchronization
        // incurs some marginal overhead in single-threaded mode but
        // we can live with it and it makes the implementation easier.
        let pool = Pool::new(1);
        pool.create_with(|vec| vec.reserve_exact(1024 * 1024));

        Self { pool }
    }

    pub(super) fn acquire_memory(&self, size: usize) -> PoolRef {
        let mut pr = self.pool.clone().get();
        pr.reserve(size);

        pr
    }

    #[must_use]
    pub(super) fn dispatch(&self, mut task: Task) -> OptionIter<io::Result<()>> {
        task.process();
        Some(task.result).into_iter()
    }
}
