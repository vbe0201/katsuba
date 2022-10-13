// A driver that sequentially processes I/O requests
// in a blocking fashion. Supported on all platforms.

use std::{fs, path::Path};

use super::Driver;

/// A blocking I/O driver that issues synchronous system
/// calls for its operations.
///
/// This makes [`Driver::wait`] a no-op since it doesn't
/// actually have anything to wait for.
#[derive(Default)]
pub struct BlockingDriver;

impl Driver for BlockingDriver {
    fn extract_file(&mut self, out: &Path, contents: &[u8]) -> anyhow::Result<()> {
        // Make sure the directory for the file exists.
        if let Some(dir) = out.parent() {
            if !dir.exists() {
                fs::create_dir_all(&dir)?;
            }
        }

        // Write the file itself.
        fs::write(out, contents)?;

        // Take care of setting correct permissions on UNIX systems.
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Some(mode) = outfile.unix_mode() {
                outfile.set_permissions(fs::Permissions::from_mode(mode))?;
            }
        }

        Ok(())
    }

    fn wait(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}
