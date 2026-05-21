use anyhow::Result;
use std::path::{Path, PathBuf};

/// On exit, machina writes its final cwd to this file. A shell wrapper reads
/// it and `cd`s there. See README for the `mc()` shell function.
pub fn target_path() -> Option<PathBuf> {
    std::env::var_os("MACHINA_CWD_FILE").map(PathBuf::from)
}

pub fn write(path: &Path) -> Result<()> {
    if let Some(out) = target_path() {
        std::fs::write(out, path.to_string_lossy().as_bytes())?;
    }
    Ok(())
}
