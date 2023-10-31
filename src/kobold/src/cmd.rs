pub mod bcd;
pub mod cs;
pub mod hash;
pub mod nav;
pub mod op;
pub mod poi;
pub mod wad;

/// Represents a command in the Kobold application.
pub trait Command {
    /// Consumes a command object and executes the handler actions
    /// associated with it.
    ///
    /// On failure, an error will be reported.
    fn handle(self) -> eyre::Result<()>;
}
