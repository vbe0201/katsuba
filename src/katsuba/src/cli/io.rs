use std::path::PathBuf;

use clap::Args;
use glob::glob;

use crate::cli::HYPHEN;

/// An input source to [`InputsOutputs`] machinery.
#[derive(Clone, Debug)]
pub enum InputSource {
    /// The input will be read from stdin.
    Stdin,
    /// The input will be read from a single file.
    File(PathBuf),
    /// Inputs will be read from multiple files (glob).
    Files(Vec<PathBuf>),
}

/// An output source to [`InputsOutputs`] machinery.
#[derive(Clone, Debug)]
pub enum OutputSource {
    /// The output will be written to stdout.
    Stdout,
    /// The output will be written to a single file.
    File(PathBuf),
    /// The output will be written to files in the directory.
    ///
    /// The extra suffix will be attached to every file.
    Dir(PathBuf, &'static str),
}

/// Generalized command options for accepting many inputs and producing
/// many outputs.
///
/// This implements all the parsing and error handling machinery to
/// avoid code repetition all over the place.
#[derive(Debug, Args)]
pub struct InputsOutputs {
    /// Specifies the input sources to process.
    ///
    /// When the value is "-", then input will be read from stdin.
    /// The semantics are the same as if a path to a single file was
    /// given instead.
    ///
    /// Everything else will be recognized as a file path. UNIX glob
    /// patterns are supported to specify many files.
    ///
    /// Note however that when a glob pattern matches more than one
    /// file, an explicit output directory for all the result files
    /// needs to be specified with the output option.
    pub input: String,

    /// An optional output source for the processed outputs.
    ///
    /// Defaults to "-" for printing output to stdout.
    ///
    /// This option takes either a path to a single file (if the
    /// input was a single file too), or a path to a directory
    /// where output files will be created for each input file.
    #[clap(short, default_value = HYPHEN)]
    pub output: PathBuf,
}

impl InputsOutputs {
    /// Evaluates the supplied arguments into input and output sources.
    pub fn evaluate(self, suffix: &'static str) -> eyre::Result<(InputSource, OutputSource)> {
        let inputs = self.input_source()?;
        let outputs = self.output_source(suffix, &inputs)?;

        Ok((inputs, outputs))
    }

    fn input_source(&self) -> eyre::Result<InputSource> {
        // First, check for a hyphen which indicates read from stdin.
        if self.input == HYPHEN {
            return Ok(InputSource::Stdin);
        }

        // Next, evaluate whatever we have as a glob pattern. Even if
        // it's just a path to a single file, it will work fine here.
        let mut paths: Vec<PathBuf> = glob(&self.input)?.collect::<Result<_, _>>()?;

        // Determine how many matches the glob produced. If it's just a
        // single file, we consider it separately because it requires
        // less clunky output handling.
        if paths.is_empty() {
            Err(eyre::eyre!(
                "failed to find files matching '{}'",
                self.input
            ))
        } else if paths.len() == 1 {
            Ok(InputSource::File(paths.remove(0)))
        } else {
            Ok(InputSource::Files(paths))
        }
    }

    fn output_source(
        self,
        suffix: &'static str,
        input: &InputSource,
    ) -> eyre::Result<OutputSource> {
        // First, check for a hyphen which indicates write to stdout.
        if self.output.as_os_str() == HYPHEN {
            return Ok(OutputSource::Stdout);
        }

        // Determine whether the output path is going to be treated as
        // a single file or as a directory based on the inputs.
        let out = match input {
            // Several input files always need to be treated as a directory output.
            InputSource::Files(..) => OutputSource::Dir(self.output, suffix),

            // Regardless of where the input comes from, if the output is
            // an existing directory we always create a new file in it.
            _ if self.output.is_dir() => OutputSource::Dir(self.output, suffix),

            // Otherwise, treat stdin and single file inputs as single file outputs.
            InputSource::Stdin | InputSource::File(..) => OutputSource::File(self.output),
        };

        Ok(out)
    }
}
