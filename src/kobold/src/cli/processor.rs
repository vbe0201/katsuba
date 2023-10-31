use std::{
    fs,
    io::{self, Read, Seek},
    path::{Path, PathBuf},
};

use eyre::Context;

use self::sealed::Missing;
use super::{InputSource, OutputSource};
use crate::{
    executor::{Buffer, Executor},
    utils,
};

mod sealed {
    pub struct Missing;
}

pub enum Reader<'a> {
    Stdin(io::Cursor<Vec<u8>>),
    File(&'a Path, io::BufReader<fs::File>),
}

impl Reader<'_> {
    pub fn get_buffer(&mut self, ex: &Executor) -> eyre::Result<Buffer<'_>> {
        match self {
            Self::Stdin(buf) => Ok(Buffer::current_borrowed(buf.get_ref())),
            Self::File(_, f) => {
                let size = f
                    .get_ref()
                    .metadata()
                    .map(|m| m.len() as usize)
                    .unwrap_or(0);

                let mut buf = ex.request_buffer(size);
                f.read_to_end(buf.as_vec())?;

                Ok(buf)
            }
        }
    }
}

impl Read for Reader<'_> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            Self::Stdin(i) => i.read(buf),
            Self::File(_, i) => i.read(buf),
        }
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        match self {
            Self::Stdin(i) => i.read_to_end(buf),
            Self::File(_, i) => i.read_to_end(buf),
        }
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        match self {
            Self::Stdin(i) => i.read_exact(buf),
            Self::File(_, i) => i.read_exact(buf),
        }
    }
}

impl Seek for Reader<'_> {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        match self {
            Self::Stdin(i) => i.seek(pos),
            Self::File(_, i) => i.seek(pos),
        }
    }

    fn stream_position(&mut self) -> io::Result<u64> {
        match self {
            Self::Stdin(i) => i.stream_position(),
            Self::File(_, i) => i.stream_position(),
        }
    }
}

pub struct Processor<R, W> {
    executor: Executor,
    reader_fn: R,
    writer_fn: W,
}

impl Processor<Missing, Missing> {
    /// Creates a new processor in uninitialized state.
    pub fn new() -> eyre::Result<Self> {
        Executor::get().map(|executor| Self {
            executor,
            reader_fn: Missing,
            writer_fn: Missing,
        })
    }

    /// Configures a callback for reading an input source into an arbitrary
    /// type for further processing.
    #[inline]
    pub fn read_with<F, T>(self, f: F) -> Processor<F, Missing>
    where
        F: FnMut(Reader<'_>, &Executor) -> eyre::Result<T>,
    {
        Processor {
            executor: self.executor,
            reader_fn: f,
            writer_fn: Missing,
        }
    }
}

impl<R, T> Processor<R, Missing>
where
    R: FnMut(Reader<'_>, &Executor) -> eyre::Result<T>,
{
    pub fn write_with<F>(self, f: F) -> Processor<R, F>
    where
        F: FnMut(&Executor, Option<PathBuf>, T, OutputSource) -> eyre::Result<()>,
    {
        Processor {
            executor: self.executor,
            reader_fn: self.reader_fn,
            writer_fn: f,
        }
    }
}

impl<R, W, T> Processor<R, W>
where
    R: FnMut(Reader<'_>, &Executor) -> eyre::Result<T>,
    W: FnMut(&Executor, Option<PathBuf>, T, OutputSource) -> eyre::Result<()>,
{
    fn stdin(&self) -> eyre::Result<Reader<'static>> {
        let mut stdin = utils::stdin_reader();

        let mut buf = io::Cursor::new(Vec::new());
        stdin.read_to_end(buf.get_mut())?;

        Ok(Reader::Stdin(buf))
    }

    fn file<'a>(&self, path: &'a Path) -> eyre::Result<Reader<'a>> {
        let file = fs::File::open(path)
            .with_context(|| format!("failed to open file '{}'", path.display()))?;

        Ok(Reader::File(path, io::BufReader::new(file)))
    }

    pub fn process(mut self, input: InputSource, output: OutputSource) -> eyre::Result<()> {
        match (input, output) {
            (InputSource::Stdin, out) => {
                let reader = self.stdin()?;

                let value = (self.reader_fn)(reader, &self.executor)?;
                (self.writer_fn)(&Executor::current(), None, value, out)
            }

            (InputSource::File(path), out) => {
                let reader = self.file(&path)?;

                let value = (self.reader_fn)(reader, &self.executor)?;
                (self.writer_fn)(&Executor::current(), Some(path), value, out)
            }

            (InputSource::Files(paths), out @ OutputSource::Dir(..)) => {
                // Dispatch work for all input paths onto the executor.
                for path in paths {
                    let reader = self.file(&path)?;
                    let value = (self.reader_fn)(reader, &self.executor)?;

                    (self.writer_fn)(&self.executor, Some(path), value, out.clone())?;
                }

                // Await the completion of all pending tasks on the executor.
                for pending in self.executor.join() {
                    pending.result?;
                }

                Ok(())
            }

            _ => unreachable!("bad state of input/output sources"),
        }
    }
}
