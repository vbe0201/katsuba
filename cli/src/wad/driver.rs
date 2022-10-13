use std::path::Path;

use anyhow::Result;

mod blocking;
pub use blocking::BlockingDriver;

/// A driver for efficient handling of I/O operations.
///
/// Implementation details depend on the target platform:
///
/// - **Windows:** `IoRing`
///
/// - **Linux:** `io_uring`
///
/// - **macOS:** `kqueue`
///
/// - **Fallback:** Sequential processing with blocking
///   syscalls on systems where the above are not supported.
pub trait Driver: Default {
    /// Issues a request for writing `contents` to the given
    /// `out` path on disk.
    fn extract_file(&mut self, out: &Path, contents: &[u8]) -> Result<()>;

    /// Waits for all pending I/O requests to complete.
    fn wait(&mut self) -> Result<()>;
}
