use std::{
    fmt, fs,
    io::{self, Read},
    path::{Path, PathBuf},
};

use either::Either;
use eyre::Context;
use rayon::prelude::*;

use super::{InputSource, OutputSource};
use crate::utils;

/// A reader over a compatible input source.
pub type Reader = Either<io::Cursor<Vec<u8>>, io::BufReader<fs::File>>;

/// Reads the whole [`Reader`] contents into a byte vector.
pub fn read_to_vec(r: Reader) -> io::Result<Vec<u8>> {
    match r {
        Either::Left(buf) => Ok(buf.into_inner()),
        Either::Right(mut f) => {
            let size = f
                .get_ref()
                .metadata()
                .map(|m| m.len() as usize)
                .unwrap_or(0);
            let mut buf = Vec::with_capacity(size);
            f.read_to_end(&mut buf)?;

            Ok(buf)
        }
    }
}

fn open_stdin() -> eyre::Result<Reader> {
    let mut stdin = utils::stdin_reader();
    let mut buf = Vec::new();
    stdin.read_to_end(&mut buf)?;

    Ok(Either::Left(io::Cursor::new(buf)))
}

fn open_file(path: &Path) -> eyre::Result<Reader> {
    let file =
        fs::File::open(path).with_context(|| format!("failed to open '{}'", path.display()))?;
    Ok(Either::Right(io::BufReader::new(file)))
}

/// Processes one or many inputs, writing each result to its matching output.
///
/// For multiple inputs, thread-level parallelism is employed to speed up
/// the processing.
///
/// The pipeline has three stages:
///
/// - `init`: Initializes resources needed for processing.
/// - `read`: Reads and processes inputs into a desirable output.
/// - `write`: Writes the outputs back.
///
/// For scenarios involving multiple inputs, the workload is partitioned
/// equally across worker threads with each thread calling `init` once
/// to construct its state before processing its chunk of inputs. `init`
/// can therefore be thought of as a thread-local state initializer.
pub fn process<T: Send, S>(
    input: InputSource,
    output: OutputSource,
    init: impl Fn() -> eyre::Result<S> + Sync,
    read: impl Fn(&mut S, Reader) -> eyre::Result<T> + Sync,
    write: impl Fn(Option<&Path>, T, OutputSource) -> eyre::Result<()> + Sync,
) -> eyre::Result<()> {
    match (input, output) {
        (InputSource::Stdin, out) => {
            let mut state = init()?;
            let value = read(&mut state, open_stdin()?)?;
            write(None, value, out)
        }

        (InputSource::File(path), out) => {
            let mut state = init()?;
            let reader = open_file(&path)?;

            read(&mut state, reader)
                .and_then(|value| write(Some(&path), value, out))
                .with_context(|| format!("while processing '{}'", path.display()))
        }

        (InputSource::Files(paths), OutputSource::Dir(dir, suffix)) => {
            fs::create_dir_all(&dir)?;

            let total = paths.len();
            let workers = rayon::current_num_threads().min(total);

            // Probe `init` once before fanning out. A broken configuration
            // makes every worker fail identically, so we check in advance
            // to produce a cleaner error message.
            init()?;

            let failures: Vec<(PathBuf, eyre::Report)> = into_round_robin_chunks(paths, workers)
                .into_par_iter()
                .map(|chunk| -> eyre::Result<Vec<_>> {
                    let mut state = init()?;
                    let mut failures = Vec::new();

                    for path in chunk {
                        let outcome =
                            open_file(&path)
                                .and_then(|r| read(&mut state, r))
                                .and_then(|v| {
                                    write(Some(&path), v, OutputSource::Dir(dir.clone(), suffix))
                                });

                        if let Err(err) = outcome {
                            failures.push((path, err));
                        }
                    }

                    Ok(failures)
                })
                .try_reduce(Vec::new, |mut acc, mut more| {
                    acc.append(&mut more);
                    Ok(acc)
                })?;

            if failures.is_empty() {
                Ok(())
            } else {
                Err(BatchError { total, failures }.into())
            }
        }

        _ => eyre::bail!("invalid input output combination"),
    }
}

fn into_round_robin_chunks<T>(items: Vec<T>, workers: usize) -> Vec<Vec<T>> {
    let cap = items.len().div_ceil(workers);

    let mut chunks: Vec<Vec<T>> = (0..workers).map(|_| Vec::with_capacity(cap)).collect();
    items
        .into_iter()
        .enumerate()
        .for_each(|(i, v)| chunks[i % workers].push(v));

    chunks
}

#[derive(Debug)]
struct BatchError {
    pub total: usize,
    pub failures: Vec<(PathBuf, eyre::Report)>,
}

impl fmt::Display for BatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const MAX_LISTED: usize = 20;

        write!(
            f,
            "{} of {} files failed to process:",
            self.failures.len(),
            self.total
        )?;

        for (path, err) in self.failures.iter().take(MAX_LISTED) {
            write!(f, "\n  - {}: {err:#}", path.display())?;
        }

        if let Some(rest) = self
            .failures
            .len()
            .checked_sub(MAX_LISTED)
            .filter(|&n| n > 0)
        {
            write!(f, "\n  ... and {rest} more")?;
        }

        Ok(())
    }
}

impl std::error::Error for BatchError {}
