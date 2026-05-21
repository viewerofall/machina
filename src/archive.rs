use anyhow::Result;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Build a sane default archive name from the targets.
pub fn default_name(targets: &[PathBuf]) -> String {
    match targets {
        [] => "archive.tar.gz".to_string(),
        [single] => {
            let stem = single
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("archive");
            format!("{}.tar.gz", stem)
        }
        _ => "archive.tar.gz".to_string(),
    }
}

/// Estimate total size of targets (walks directories).
pub fn estimate_size(targets: &[PathBuf]) -> u64 {
    targets.iter().map(|p| size_of(p)).sum()
}

fn size_of(p: &Path) -> u64 {
    let Ok(meta) = std::fs::metadata(p) else {
        return 0;
    };
    if meta.is_file() {
        return meta.len();
    }
    if meta.is_dir() {
        let mut total = 0u64;
        if let Ok(entries) = std::fs::read_dir(p) {
            for e in entries.flatten() {
                total = total.saturating_add(size_of(&e.path()));
            }
        }
        return total;
    }
    0
}

/// Create a .tar.gz archive containing the given paths. Runs synchronously.
pub fn create(name: &str, cwd: &Path, targets: &[PathBuf]) -> Result<()> {
    if targets.is_empty() {
        anyhow::bail!("no files to archive");
    }

    let out = if Path::new(name).is_absolute() {
        PathBuf::from(name)
    } else {
        cwd.join(name)
    };

    let mut cmd = Command::new("tar");
    cmd.arg("-czf").arg(&out).current_dir(cwd);

    for t in targets {
        // Pass relative path if possible (cleaner archive structure)
        let arg: PathBuf = match t.strip_prefix(cwd) {
            Ok(rel) => rel.to_path_buf(),
            Err(_) => t.clone(),
        };
        cmd.arg(arg);
    }

    let status = cmd.status()?;
    if !status.success() {
        anyhow::bail!("tar exited with non-zero status");
    }
    Ok(())
}
