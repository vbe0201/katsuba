use std::{
    io,
    sync::{mpsc, Arc},
};

use enum_map::{enum_map, Enum, EnumMap};
use threadpool::{Builder, ThreadPool};

use super::Task;
use crate::memory::{Pool, PoolRef};

const WORKER_NAME: &str = "katsuba-worker";
const WORKER_STACK: usize = 1_048_576;

// An arbitrary threshold to limit the amount of pending tasks in
// the executor. This prevents exhausting available file handles
// on Linux when too much concurrent work gets generated.
const QUEUE_THRESHOLD: usize = 8;

fn make_worker_pool(nthreads: usize) -> ThreadPool {
    Builder::new()
        .num_threads(nthreads)
        .thread_name(WORKER_NAME.into())
        .thread_stack_size(WORKER_STACK)
        .build()
}

#[inline]
const fn bucket_capacity(nthreads: usize) -> usize {
    // We choose the upper bound under the assumption that no buffers
    // get re-used in-between. Every running background thread would
    // have its own buffer, plus the number of pending tasks, which
    // we limit.
    nthreads + QUEUE_THRESHOLD
}

#[derive(Clone, Copy, Debug, PartialEq, Enum)]
enum BucketSize {
    FourK,
    EightK,
    OneM,
    EightM,
    SixteenM,
    Unbounded,
}

struct Bucket {
    pool: Arc<Pool>,
    size: usize,
}

impl Bucket {
    fn new(capacity: usize, bucket_size: usize) -> Self {
        Self {
            pool: Pool::new(capacity),
            size: bucket_size,
        }
    }
}

enum Notification {
    Done(io::Result<()>),
    End,
}

/// An executor flavor which processes tasks on background threads.
pub struct Threaded {
    pool: ThreadPool,
    tx: mpsc::Sender<Notification>,
    rx: mpsc::Receiver<Notification>,
    memory_buckets: EnumMap<BucketSize, Bucket>,
}

impl Threaded {
    pub(super) fn new(nthreads: usize) -> Self {
        let (tx, rx) = mpsc::channel();

        let bucket_cap = bucket_capacity(nthreads);
        let memory_buckets = enum_map! {
            BucketSize::FourK => Bucket::new(bucket_cap, 4096),
            BucketSize::EightK => Bucket::new(bucket_cap, 8192),
            BucketSize::OneM => Bucket::new(bucket_cap, 1024 * 1024),
            BucketSize::EightM => Bucket::new(bucket_cap, 8 * 1024 * 1024),
            BucketSize::SixteenM => Bucket::new(bucket_cap, 16 * 1024 * 1024),
            // The unbounded bucket supports arbitrary growth. It doesn't
            // preallocate memory in advance because it's used in rare
            // situations where 16M don't suffice.
            BucketSize::Unbounded => Bucket::new(bucket_cap, 0),
        };

        // Ensure that we have at least one buffer of each size so
        // that we can always make forward progress.
        for (_, bucket) in &memory_buckets {
            bucket
                .pool
                .create_with(|vec| vec.reserve_exact(bucket.size));
        }

        Self {
            pool: make_worker_pool(nthreads),
            tx,
            rx,
            memory_buckets,
        }
    }

    fn find_bucket(&self, size: usize) -> BucketSize {
        let mut bucket_size = BucketSize::FourK;
        for (next_bucket_size, bucket) in &self.memory_buckets {
            bucket_size = next_bucket_size;
            if bucket.size >= size {
                break;
            }
        }

        let bucket = &self.memory_buckets[bucket_size];
        assert!(
            bucket_size == BucketSize::Unbounded || size <= bucket.size,
            "cannot find a bucket for '{size}' bytes"
        );

        bucket_size
    }

    pub(super) fn acquire_memory(&self, size: usize) -> PoolRef {
        let bucket_size = self.find_bucket(size);
        let bucket = &self.memory_buckets[bucket_size];

        // `size` always fits into the selected bucket when we make it
        // here, so just allocate the full bucket size for the buffer.
        // This may be a no-op for pre-allocated buffers.
        let mut pr = bucket.pool.clone().get();
        pr.reserve_exact(bucket.size);

        pr
    }

    pub(super) fn execute(&self, mut task: Task) {
        let tx = self.tx.clone();
        self.pool.execute(move || {
            task.process();
            let _ = tx.send(Notification::Done(task.result));
        });
    }

    #[must_use]
    pub(super) fn dispatch(&self, task: Task) -> SubmitIterator<'_> {
        SubmitIterator {
            threaded: self,
            task: Some(task),
        }
    }

    #[must_use]
    pub(super) fn join(&self) -> JoinIterator<'_> {
        self.pool.join();
        let _ = self.tx.send(Notification::End);

        JoinIterator { threaded: self }
    }
}

impl Drop for Threaded {
    fn drop(&mut self) {
        self.join().for_each(drop);
    }
}

pub struct SubmitIterator<'a> {
    threaded: &'a Threaded,
    task: Option<Task>,
}

impl Iterator for SubmitIterator<'_> {
    type Item = io::Result<()>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.threaded.pool.queued_count() < QUEUE_THRESHOLD {
            if let Some(t) = self.task.take() {
                self.threaded.execute(t);
            }

            None
        } else {
            for notification in self.threaded.rx.iter() {
                if let Notification::Done(t) = notification {
                    return Some(t);
                }

                if self.threaded.pool.queued_count() < QUEUE_THRESHOLD {
                    if let Some(t) = self.task.take() {
                        self.threaded.execute(t);
                    }

                    return None;
                }
            }

            unreachable!()
        }
    }
}

pub struct JoinIterator<'a> {
    threaded: &'a Threaded,
}

impl Iterator for JoinIterator<'_> {
    type Item = io::Result<()>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.threaded.rx.recv() {
            Ok(notification) => match notification {
                Notification::Done(t) => Some(t),
                Notification::End => None,
            },

            Err(_) => None,
        }
    }
}
