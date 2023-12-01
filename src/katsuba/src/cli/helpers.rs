use std::path::PathBuf;

use katsuba_executor::Executor;

use super::OutputSource;
use crate::utils;

/// Helper function to be used with [`Executor::write_with`] for mapping
/// any serializable `T` value to an output source.
pub fn write_as_json<T: serde::Serialize>(
    ex: &Executor,
    inpath: Option<PathBuf>,
    value: T,
    out: OutputSource,
) -> eyre::Result<()> {
    match (out, inpath) {
        (OutputSource::Stdout, _) => utils::serialize_to_output_source(ex, None, &value),
        (OutputSource::File(path), _) => utils::serialize_to_output_source(ex, Some(path), &value),
        (OutputSource::Dir(mut out, suffix), Some(path)) => {
            // Create a file named after the input in the output directory.
            let infile = path.with_extension(suffix);
            out.push(infile.file_name().unwrap());

            utils::serialize_to_output_source(ex, Some(out), &value)
        }

        (OutputSource::Dir(..), None) => Err(eyre::eyre!(
            "output path for stdin input is directory; specify a file path instead"
        )),
    }
}
