use std::{
    collections::HashSet,
    env, fs,
    path::{Path, PathBuf},
};

use eyre::Context;
use kobold_wad::{Archive, Inflater};

use crate::{
    cli::OutputSource,
    executor::{Buffer, Executor, Task},
};

struct SafeArchiveDrop<'a> {
    ex: &'a Executor,
    archive: Archive,
}

impl Drop for SafeArchiveDrop<'_> {
    fn drop(&mut self) {
        // Join all pending tasks on the executor to make sure none of
        // them hold onto dangling `archive` references anymore after
        // dropping it.
        self.ex.join().for_each(drop);
    }
}

fn fetch_file_contents<'a>(
    ex: &Executor,
    archive: &'a Archive,
    inflater: &mut Inflater,
    file: &kobold_wad::types::File,
) -> eyre::Result<Buffer<'a>> {
    let contents = archive.file_contents(file);
    let buffer = match file.compressed {
        true => {
            let len = file.uncompressed_size as usize;

            let mut buf = ex.request_buffer(len);
            buf.as_vec().resize(len, 0);

            inflater.decompress_into(&mut buf, contents)?;
            buf
        }
        false => Buffer::current_borrowed(contents),
    };

    Ok(buffer)
}

fn create_directory_tree(archive: &Archive, out: &Path) -> eyre::Result<()> {
    // Pre-compute the directory structure we need to create.
    let mut out_paths = HashSet::new();
    for file in archive.files().keys() {
        let file: &Path = file.as_ref();

        if let Some(p) = file.parent() {
            out_paths.insert(p);

            // Check for parent of parent so that we can remove
            // entries which would just need unnecessary syscalls.
            if let Some(p) = p.parent() {
                out_paths.remove(p);
            }
        }
    }

    // Create all the directories with minimal required syscalls.
    // This has shown to make a drastic performance difference on Windows.
    for path in out_paths {
        let path = out.join(path);
        fs::create_dir_all(&path)
            .with_context(|| format!("failed to create directory '{}'", path.display()))?;
    }

    Ok(())
}

pub fn extract_archive(
    ex: &Executor,
    inpath: Option<PathBuf>,
    archive: Archive,
    out: OutputSource,
) -> eyre::Result<()> {
    // Determine the output directory for the archive files.
    // Since we can't print here, we use the cwd instead.
    let input_stem = inpath.as_ref().and_then(|p| p.file_stem()).unwrap();
    let mut out = match out {
        OutputSource::Stdout => env::current_dir()?,
        OutputSource::File(p) | OutputSource::Dir(p, ..) => p,
    };
    out.push(input_stem);

    // First, create all the directories for the output files.
    create_directory_tree(&archive, &out)?;

    // This guard ensures we can safely share references into `archive`
    // with the pool without risking dangling in the case of an error.
    let sad = SafeArchiveDrop { ex, archive };
    let mode = sad.archive.mode();

    // Next, we do the extraction of data out of the archive on the
    // current thread while simultaneously dispatching the file I/O
    // operations to the executor.
    let mut inflater = Inflater::new();
    for (path, file) in sad.archive.files() {
        let path = out.join(path);

        // SAFETY: We can never end up with dangling references into
        // `archive` because `sad` joins all pending tasks on drop.
        let buffer = fetch_file_contents(ex, &sad.archive, &mut inflater, file)?;
        let buffer = unsafe { buffer.extend_lifetime() };

        let task = Task::create_file(path, buffer, mode);
        for pending in ex.dispatch(task) {
            pending.result?;
        }
    }

    Ok(())
}
