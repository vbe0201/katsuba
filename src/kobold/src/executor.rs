use std::{
    env, fs,
    io::{self, Write},
    option::IntoIter as OptionIter,
    path::{Path, PathBuf},
    thread,
};

use eyre::Context;

mod buffer;
pub use buffer::*;

mod current;
use current::Current;

mod threaded;
use threaded::Threaded;

const KOBOLD_WORKER_THREADS: &str = "KOBOLD_WORKER_THREADS";

fn available_threads() -> eyre::Result<usize> {
    match env::var(KOBOLD_WORKER_THREADS) {
        Ok(value) => value.parse::<usize>().with_context(|| {
            format!(
                "invalid value in {}; must be natural number",
                KOBOLD_WORKER_THREADS
            )
        }),

        Err(_) => Ok(thread::available_parallelism()
            .map(|p| p.get())
            .unwrap_or(1)),
    }
}

/// A task to carry out inside the executor.
pub struct Task {
    pub path: PathBuf,
    pub kind: Kind,
    pub result: io::Result<()>,
}

/// Types of work to process.
pub enum Kind {
    CreateFile {
        contents: Buffer<'static>,
        mode: u32,
    },
}

impl Task {
    /// Creates a task to create a new file.
    pub fn create_file(path: PathBuf, contents: Buffer<'static>, mode: u32) -> Self {
        Self {
            path,
            kind: Kind::CreateFile { contents, mode },
            result: Ok(()),
        }
    }

    pub(super) fn process(&mut self) {
        match &mut self.kind {
            Kind::CreateFile { contents, mode } => {
                contents.clear();
                self.result = write_file(&self.path, contents, *mode);
            }
        }
    }
}

pub(super) fn write_file(path: &Path, contents: &[u8], _mode: u32) -> io::Result<()> {
    let mut opts = fs::OpenOptions::new();

    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(_mode);
    }

    let mut file = opts.write(true).create(true).truncate(true).open(path)?;
    file.write_all(contents)?;

    Ok(())
}

pub enum Executor {
    Current(Current),
    Threaded(Threaded),
}

impl Executor {
    /// Creates a single-threaded executor on the current thread.
    #[inline]
    pub fn current() -> Self {
        Self::Current(Current)
    }

    /// Gets the preferred executor for the number of available
    /// worker threads on the system.
    pub fn get() -> eyre::Result<Self> {
        match available_threads()? {
            0 | 1 => Ok(Self::Current(Current)),
            n => Ok(Self::Threaded(Threaded::new(n))),
        }
    }

    /// Requests an in-memory buffer for the executor to use.
    pub fn request_buffer(&self, capacity: usize) -> Buffer<'static> {
        match self {
            Self::Threaded(t) => Buffer::pooled(t.acquire_memory(capacity)),
            Self::Current(..) => Buffer::current_owned(Vec::with_capacity(capacity)),
        }
    }

    /// Dispatches a task to be performed inside the executor.
    #[must_use = "Iterator must be consumed to ensure the task gets dispatched"]
    pub fn dispatch(&self, task: Task) -> SubmitIterator<'_> {
        match self {
            Self::Threaded(t) => SubmitIterator::Threaded(t.dispatch(task)),
            Self::Current(c) => SubmitIterator::Current(c.dispatch(task)),
        }
    }

    /// Waits for the completion of all tasks in the executor.
    #[must_use = "Iterator must be consumed to ensure all tasks finish"]
    pub fn join(&self) -> JoinIterator<'_> {
        match self {
            Self::Threaded(t) => JoinIterator::Threaded(t.join()),
            Self::Current(..) => JoinIterator::Current,
        }
    }
}

/// An iterator that yields finished tasks until a slot for the pending
/// task becomes available.
pub enum SubmitIterator<'a> {
    Current(OptionIter<Task>),
    Threaded(threaded::SubmitIterator<'a>),
}

impl Iterator for SubmitIterator<'_> {
    type Item = Task;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Threaded(si) => si.next(),
            Self::Current(oi) => oi.next(),
        }
    }
}

/// An iterator that yields finished tasks until no more are running on
/// the executor.
pub enum JoinIterator<'a> {
    Current,
    Threaded(threaded::JoinIterator<'a>),
}

impl Iterator for JoinIterator<'_> {
    type Item = Task;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Threaded(ji) => ji.next(),
            Self::Current => None,
        }
    }
}
