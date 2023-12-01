use std::{
    fs,
    io::{self, Read, Seek},
    path::{Path, PathBuf},
};

use eyre::Context;
use katsuba_executor::{Buffer, Executor};

use self::sealed::Missing;
use super::{InputSource, OutputSource};
use crate::utils;

mod sealed {
    pub struct Missing;
}

/// A [`Read`]er over a compatible input source.
pub enum Reader<'a> {
    Stdin(io::Cursor<Vec<u8>>),
    File(&'a Path, io::BufReader<fs::File>),
}

impl Reader<'_> {
    /// Gets the data in the reader as a [`Buffer`], if possible.
    pub fn get_buffer(&mut self, ex: &Executor) -> eyre::Result<Buffer<'_>> {
        match self {
            Self::Stdin(buf) => Ok(Buffer::borrowed(buf.get_ref())),
            Self::File(_, f) => {
                let size = f
                    .get_ref()
                    .metadata()
                    .map(|m| m.len() as usize)
                    .unwrap_or(0);

                ex.request_buffer(size, |buf| {
                    f.read_to_end(buf)?;
                    Ok(())
                })
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

/// A bias to hint to the [`Processor`] which executor type should
/// be preferred for workloads consisting of a single input.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bias {
    Current,
    Threaded,
}

/// Processes input sources and maps them to output sources.
pub struct Processor<R, W> {
    bias: Bias,
    reader_fn: R,
    writer_fn: W,
}

impl Processor<Missing, Missing> {
    /// Creates a new processor in uninitialized state.
    ///
    /// The bias nudges the processor towards which executor to use for
    /// single-file workloads. Workloads of many files will always use
    /// a threaded executor, if available, regardless of bias.
    pub fn new(bias: Bias) -> eyre::Result<Self> {
        Ok(Self {
            bias,
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
            bias: self.bias,
            reader_fn: f,
            writer_fn: Missing,
        }
    }
}

impl<R, T> Processor<R, Missing>
where
    R: FnMut(Reader<'_>, &Executor) -> eyre::Result<T>,
{
    /// Configures a callback for writing an element to an output source.
    pub fn write_with<F>(self, f: F) -> Processor<R, F>
    where
        F: FnMut(&Executor, Option<PathBuf>, T, OutputSource) -> eyre::Result<()>,
    {
        Processor {
            bias: self.bias,
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

    /// Processes the given input source into the given output source.
    ///
    /// Depending on the configuration, this may use single-threaded or
    /// multi-threaded I/O for processing.
    pub fn process(mut self, input: InputSource, output: OutputSource) -> eyre::Result<()> {
        let mut executor = match self.bias {
            Bias::Current => Executor::current(),
            Bias::Threaded => Executor::get()?,
        };

        match (input, output) {
            (InputSource::Stdin, out) => {
                let reader = self.stdin()?;

                let value = (self.reader_fn)(reader, &executor)?;
                (self.writer_fn)(&mut executor, None, value, out)
            }

            (InputSource::File(path), out) => {
                let reader = self.file(&path)?;

                let value = (self.reader_fn)(reader, &executor)?;
                (self.writer_fn)(&mut executor, Some(path), value, out)
            }

            (InputSource::Files(paths), OutputSource::Dir(out, suffix)) => {
                // When processing multiple input files, we ignore the bias.
                if let Bias::Current = self.bias {
                    executor = Executor::get()?;
                }

                // Create the specified out directory if it doesn't exist.
                fs::create_dir_all(&out)?;

                // Dispatch work for all input paths onto the executor.
                for path in paths {
                    let reader = self.file(&path)?;
                    let value = (self.reader_fn)(reader, &executor)?;

                    (self.writer_fn)(
                        &mut executor,
                        Some(path),
                        value,
                        OutputSource::Dir(out.clone(), suffix),
                    )?;
                }

                // Await the completion of all pending tasks on the executor.
                for pending in executor.join() {
                    pending?;
                }

                Ok(())
            }

            _ => unreachable!("bad state of input/output sources"),
        }
    }
}
