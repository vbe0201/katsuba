use std::{
    mem,
    sync::{mpsc, Arc},
};

use enum_map::{enum_map, Enum, EnumMap};
use threadpool::{Builder, ThreadPool};

use super::{PoolRef, Task};

const WORKER_NAME: &str = "katsuba-worker";
const WORKER_STACK: usize = 1_048_576;

fn make_worker_pool(nthreads: usize) -> ThreadPool {
    Builder::new()
        .num_threads(nthreads)
        .thread_name(WORKER_NAME.into())
        .thread_stack_size(WORKER_STACK)
        .build()
}

#[derive(Clone, Copy, Debug, Enum)]
enum Bucket {
    FourK,
    EightK,
    OneM,
    EightM,
    SixteenM,
}

enum Notification {
    Done(Task),
    End,
}

struct Pool {
    pool: Arc<sharded_slab::Pool<Vec<u8>>>,
    size: usize,
}

impl Pool {
    fn new(size: usize) -> Self {
        Self {
            pool: Arc::new(sharded_slab::Pool::new()),
            size,
        }
    }
}

struct MemoryPool {
    inner: EnumMap<Bucket, Pool>,
}

impl MemoryPool {
    fn new() -> Self {
        let inner = enum_map! {
            Bucket::FourK => Pool::new(4096),
            Bucket::EightK => Pool::new(8192),
            Bucket::OneM => Pool::new(1024 * 1024),
            Bucket::EightM => Pool::new(8 * 1024 * 1024),
            Bucket::SixteenM => Pool::new(16 * 1024 * 1024),
        };

        // Ensure that we have at least one pool of each size so
        // we can always make forward progress.
        for (_, pool) in &inner {
            let key = pool
                .pool
                .create_with(|vec| vec.reserve_exact(pool.size - vec.len()))
                .unwrap();
            pool.pool.clear(key);
        }

        Self { inner }
    }

    fn find_bucket(&self, capacity: usize) -> Bucket {
        let mut bucket = Bucket::FourK;
        for (next_bucket, pool) in &self.inner {
            bucket = next_bucket;
            if pool.size >= capacity {
                break;
            }
        }

        let pool = &self.inner[bucket];
        assert!(capacity <= pool.size);

        bucket
    }

    fn acquire_memory(&self, capacity: usize) -> PoolRef {
        let bucket = self.find_bucket(capacity);
        let pool = &self.inner[bucket];

        let mut orm = pool.pool.clone().create_owned().unwrap();
        orm.reserve_exact(pool.size);

        PoolRef::Mut(orm, pool.pool.clone())
    }
}

pub struct Threaded {
    pool: ThreadPool,
    tx: mpsc::Sender<Notification>,
    rx: mpsc::Receiver<Notification>,
    memory_pool: MemoryPool,
}

impl Threaded {
    pub fn new(nthreads: usize) -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            pool: make_worker_pool(nthreads),
            tx,
            rx,
            memory_pool: MemoryPool::new(),
        }
    }

    fn execute(&self, mut task: Task) {
        let tx = self.tx.clone();

        self.pool.execute(move || {
            task.process();
            let _ = tx.send(Notification::Done(task));
        });
    }

    pub fn acquire_memory(&self, capacity: usize) -> PoolRef {
        self.memory_pool.acquire_memory(capacity)
    }

    #[must_use]
    pub fn dispatch(&self, task: Task) -> SubmitIterator<'_> {
        SubmitIterator {
            threaded: self,
            task: Some(task),
        }
    }

    #[must_use]
    pub fn join(&self) -> JoinIterator<'_> {
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
    type Item = Task;

    fn next(&mut self) -> Option<Self::Item> {
        // Arbitrary threshold which prevents exhausting available file
        // handles for the process while still being able to generate
        // reasonable workloads onto the pool from the main thread.
        let threshold = 8;

        if self.threaded.pool.queued_count() < threshold {
            if let Some(t) = mem::take(&mut self.task) {
                self.threaded.execute(t);
            }

            None
        } else {
            for notification in self.threaded.rx.iter() {
                if let Notification::Done(t) = notification {
                    return Some(t);
                }

                if self.threaded.pool.queued_count() < threshold {
                    if let Some(t) = mem::take(&mut self.task) {
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
    type Item = Task;

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
