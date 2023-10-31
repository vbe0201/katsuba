use clap::{ArgAction, Args};

/// Configures the verbosity of the builtin logger.
#[derive(Clone, Copy, Debug, Args)]
pub struct Verbosity {
    /// Configures the log verbosity of Kobold.
    ///
    /// `-v` is Debug, `-vv` is Trace.
    #[clap(short, long, action = ArgAction::Count, global = true)]
    pub verbose: u8,
}

impl Verbosity {
    /// Configures the global logger based on the settings.
    pub fn setup(self) {
        let level = self.log_level();
        simple_logger::init_with_level(level).unwrap();
    }

    fn log_level(self) -> log::Level {
        match self.verbose {
            0 => log::Level::Info,
            1 => log::Level::Debug,
            _ => log::Level::Trace,
        }
    }
}
