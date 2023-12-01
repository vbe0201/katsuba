use std::{env, io, option::IntoIter as OptionIter, path::PathBuf, thread};

use thiserror::Error;

use crate::memory::Buffer;

mod current;
use current::Current;

mod r#impl;

mod threaded;
use threaded::Threaded;

const KATSUBA_WORKER_THREADS: &str = "KATSUBA_WORKER_THREADS";

#[derive(Clone, Debug, Error)]
#[error(
    "invalid value in {}; must be a natural number",
    KATSUBA_WORKER_THREADS
)]
pub struct BadConfiguration;

fn available_threads() -> Result<usize, BadConfiguration> {
    match env::var(KATSUBA_WORKER_THREADS) {
        Ok(value) => value.parse().map_err(|_| BadConfiguration),

        Err(_) => Ok(thread::available_parallelism()
            .map(|p| p.get())
            .unwrap_or(1)),
    }
}

/// A task to carry out inside the executor.
///
/// Tasks are constructed by the user and dispatched to the
/// threadpool. Task results will be returned back to the
/// caller.
#[derive(Debug)]
pub struct Task {
    /// The relevant path for the I/O operation.
    pub path: PathBuf,
    /// The specific type of operation to perform.
    pub kind: TaskKind,
    /// The outcome of the operation, set after completion.
    pub result: io::Result<()>,
}

/// Types of I/O to process on the worker threads.
#[derive(Debug)]
pub enum TaskKind {
    /// Creates a new file at the given path with specified contents.
    ///
    /// Optionally, on UNIX platforms, a file mode choice may be
    /// respected by the platform.
    CreateFile {
        contents: Buffer<'static>,
        mode: u32,
    },

    /// Creates a directory from the given path.
    ///
    /// This will also create all subdirectories.
    CreateDir,
}

impl Task {
    /// Creates a [`Task`] for making a new file.
    pub fn create_file(path: PathBuf, contents: Buffer<'static>, mode: u32) -> Self {
        Self {
            path,
            kind: TaskKind::CreateFile { contents, mode },
            result: Ok(()),
        }
    }

    /// Creates a [`Task`] for making new directories.
    pub fn create_dir(path: PathBuf) -> Self {
        Self {
            path,
            kind: TaskKind::CreateDir,
            result: Ok(()),
        }
    }

    pub(super) fn process(&mut self) {
        match &mut self.kind {
            TaskKind::CreateFile { contents, mode } => {
                self.result = r#impl::write_file(&self.path, contents, *mode);
            }

            TaskKind::CreateDir => {
                self.result = r#impl::create_dir(&self.path);
            }
        }
    }
}

/// An executor for file I/O processing.
///
/// Configuration is possible with the `KATSUBA_WORKER_THREADS`
/// environment variable specifying the number of threads to use.
/// If not set, falls back to [`thread::available_parallelism`].
///
/// The API is the same for both flavors of execution and users
/// should not need to worry about any execution flavor details.
pub enum Executor {
    /// A single-threaded executor on the current thread.
    Current(Current),
    /// A multithreaded executor performing work on background threads.
    Threaded(Threaded),
}

impl Executor {
    /// Creates a single-threaded executor on the current thread.
    #[inline]
    pub fn current() -> Self {
        Self::Current(Current::new())
    }

    /// Gets the preferred executor for the configuration of available
    /// worker threads on the system.
    #[inline]
    pub fn get() -> Result<Self, BadConfiguration> {
        match available_threads()? {
            0 | 1 => Ok(Self::current()),
            n => Ok(Self::Threaded(Threaded::new(n))),
        }
    }

    /// Requests an in-memory buffer for I/O from the executor.
    ///
    /// `f` takes a vector reference with capacity for at least `size`
    /// bytes and populates it. Based on the returned result, a [`Buffer`]
    /// over said vector or the error will be handed to the caller.
    pub fn request_buffer<F, E>(&self, size: usize, f: F) -> Result<Buffer<'static>, E>
    where
        F: FnOnce(&mut Vec<u8>) -> Result<(), E>,
    {
        let mut pr = match self {
            Self::Threaded(t) => t.acquire_memory(size),
            Self::Current(c) => c.acquire_memory(size),
        };

        debug_assert!(pr.is_empty());
        f(&mut pr)?;

        Ok(Buffer::pooled(pr))
    }

    /// Dispatches a task to be performed inside the executor.
    pub fn dispatch(&self, task: Task) -> SubmitIterator<'_> {
        match self {
            Self::Threaded(t) => SubmitIterator::Threaded(t.dispatch(task)),
            Self::Current(c) => SubmitIterator::Current(c.dispatch(task)),
        }
    }

    /// Joins all pending tasks on the executor.
    pub fn join(&self) -> JoinIterator<'_> {
        match self {
            Self::Threaded(t) => JoinIterator::Threaded(t.join()),
            Self::Current(..) => JoinIterator::Current,
        }
    }
}

/// An iterator that yields finished tasks until resources become
/// available to enqueue the pending task.
#[must_use = "Consume this Iterator to ensure the pending task gets executed"]
pub enum SubmitIterator<'a> {
    Current(OptionIter<io::Result<()>>),
    Threaded(threaded::SubmitIterator<'a>),
}

impl Iterator for SubmitIterator<'_> {
    type Item = io::Result<()>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Threaded(si) => si.next(),
            Self::Current(oi) => oi.next(),
        }
    }
}

/// An iterator that yields finished tasks until no more are running
/// on the executor.
#[must_use = "Consume this Iterator to ensure all tasks have terminated"]
pub enum JoinIterator<'a> {
    Current,
    Threaded(threaded::JoinIterator<'a>),
}

impl Iterator for JoinIterator<'_> {
    type Item = io::Result<()>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Threaded(ji) => ji.next(),
            Self::Current => None,
        }
    }
}
