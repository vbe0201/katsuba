use std::{
    fs,
    io::{self, Read, Seek},
    path::{Path, PathBuf},
};

use eyre::Context;
use rayon::prelude::*;

use super::{InputSource, OutputSource};
use crate::utils;

/// A reader over a compatible input source.
pub enum Reader {
    Stdin(io::Cursor<Vec<u8>>),
    File(io::BufReader<fs::File>),
}

impl Reader {
    pub fn read_to_vec(self) -> io::Result<Vec<u8>> {
        match self {
            Self::Stdin(buf) => Ok(buf.into_inner()),
            Self::File(mut f) => {
                let size = f
                    .get_ref()
                    .metadata()
                    .map(|m| m.len() as usize)
                    .unwrap_or(0);
                let mut buf = Vec::with_capacity(size);
                f.read_to_end(&mut buf)?;

                #[cfg(windows)]
                blocking::unblock(move || drop(f)).detach();

                Ok(buf)
            }
        }
    }
}

impl Read for Reader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            Self::Stdin(i) => i.read(buf),
            Self::File(i) => i.read(buf),
        }
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        match self {
            Self::Stdin(i) => i.read_to_end(buf),
            Self::File(i) => i.read_to_end(buf),
        }
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        match self {
            Self::Stdin(i) => i.read_exact(buf),
            Self::File(i) => i.read_exact(buf),
        }
    }
}

impl Seek for Reader {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        match self {
            Self::Stdin(i) => i.seek(pos),
            Self::File(i) => i.seek(pos),
        }
    }

    fn stream_position(&mut self) -> io::Result<u64> {
        match self {
            Self::Stdin(i) => i.stream_position(),
            Self::File(i) => i.stream_position(),
        }
    }
}

fn open_stdin() -> eyre::Result<Reader> {
    let mut stdin = utils::stdin_reader();
    let mut buf = Vec::new();
    stdin.read_to_end(&mut buf)?;

    Ok(Reader::Stdin(io::Cursor::new(buf)))
}

fn open_file(path: &Path) -> eyre::Result<Reader> {
    let file =
        fs::File::open(path).with_context(|| format!("failed to open '{}'", path.display()))?;
    Ok(Reader::File(io::BufReader::new(file)))
}

/// Processes inputs sequentially. Use for commands that handle parallelism internally.
pub fn process<T>(
    input: InputSource,
    output: OutputSource,
    mut read: impl FnMut(Reader) -> eyre::Result<T>,
    mut write: impl FnMut(Option<PathBuf>, T, OutputSource) -> eyre::Result<()>,
) -> eyre::Result<()> {
    match (input, output) {
        (InputSource::Stdin, out) => {
            let value = read(open_stdin()?)?;
            write(None, value, out)
        }

        (InputSource::File(path), out) => {
            let value = read(open_file(&path)?)?;
            write(Some(path), value, out)
        }

        (InputSource::Files(paths), OutputSource::Dir(dir, suffix)) => {
            fs::create_dir_all(&dir)?;
            for path in paths {
                let value = read(open_file(&path)?)?;
                write(Some(path), value, OutputSource::Dir(dir.clone(), suffix))?;
            }

            Ok(())
        }

        _ => unreachable!("invalid input/output combination"),
    }
}

/// Processes inputs with rayon parallelism for batch operations.
pub fn process_par<T, S, R, W>(
    input: InputSource,
    output: OutputSource,
    init: impl Fn() -> S + Sync + Send,
    read: R,
    write: W,
) -> eyre::Result<()>
where
    T: Send,
    S: Send,
    R: Fn(&mut S, Reader) -> eyre::Result<T> + Sync,
    W: Fn(Option<PathBuf>, T, OutputSource) -> eyre::Result<()> + Sync,
{
    match (input, output) {
        (InputSource::Stdin, out) => {
            let value = read(&mut init(), open_stdin()?)?;
            write(None, value, out)
        }

        (InputSource::File(path), out) => {
            let value = read(&mut init(), open_file(&path)?)?;
            write(Some(path), value, out)
        }

        (InputSource::Files(paths), OutputSource::Dir(dir, suffix)) => {
            fs::create_dir_all(&dir)?;
            paths
                .into_par_iter()
                .try_for_each_init(init, |state, path| {
                    let value = read(state, open_file(&path)?)?;
                    write(Some(path), value, OutputSource::Dir(dir.clone(), suffix))
                })
        }

        _ => unreachable!("invalid input/output combination"),
    }
}
