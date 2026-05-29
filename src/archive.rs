use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::fs;

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

/// Check if a file is a supported archive format.
pub fn is_archive(path: &Path) -> bool {
    let path_str = path.to_string_lossy().to_lowercase();
    path_str.ends_with(".tar.gz")
        || path_str.ends_with(".tgz")
        || path_str.ends_with(".tar.bz2")
        || path_str.ends_with(".tbz2")
        || path_str.ends_with(".tar.xz")
        || path_str.ends_with(".txz")
        || path_str.ends_with(".tar")
        || path_str.ends_with(".zip")
        || path_str.ends_with(".7z")
        || path_str.ends_with(".rar")
}

/// Detect archive type from file extension.
fn detect_type(path: &Path) -> Result<&'static str> {
    let path_str = path.to_string_lossy().to_lowercase();
    if path_str.ends_with(".tar.gz") || path_str.ends_with(".tgz") {
        Ok("tar.gz")
    } else if path_str.ends_with(".tar.bz2") || path_str.ends_with(".tbz2") {
        Ok("tar.bz2")
    } else if path_str.ends_with(".tar.xz") || path_str.ends_with(".txz") {
        Ok("tar.xz")
    } else if path_str.ends_with(".tar") {
        Ok("tar")
    } else if path_str.ends_with(".zip") {
        Ok("zip")
    } else if path_str.ends_with(".7z") {
        Ok("7z")
    } else if path_str.ends_with(".rar") {
        Ok("rar")
    } else {
        Err(anyhow!("unsupported archive format"))
    }
}

/// Extract archive to destination directory. Runs synchronously.
pub fn extract(archive: &Path, dest: &Path) -> Result<()> {
    if !archive.exists() {
        return Err(anyhow!("archive not found"));
    }
    if !dest.exists() {
        fs::create_dir_all(dest)?;
    }

    let archive_type = detect_type(archive)?;

    match archive_type {
        "tar.gz" | "tgz" | "tar.bz2" | "tbz2" | "tar.xz" | "txz" | "tar" => {
            let flag = match archive_type {
                "tar.gz" | "tgz" => "-xzf",
                "tar.bz2" | "tbz2" => "-xjf",
                "tar.xz" | "txz" => "-xJf",
                "tar" => "-xf",
                _ => unreachable!(),
            };
            let status = Command::new("tar")
                .arg(flag)
                .arg(archive)
                .current_dir(dest)
                .status()?;
            if !status.success() {
                return Err(anyhow!("tar extraction failed"));
            }
        }
        "zip" => {
            let status = Command::new("unzip")
                .arg("-q")
                .arg(archive)
                .current_dir(dest)
                .status()?;
            if !status.success() {
                return Err(anyhow!("unzip failed"));
            }
        }
        "7z" => {
            let status = Command::new("7z")
                .arg("x")
                .arg(archive)
                .arg(format!("-o{}", dest.display()))
                .status()?;
            if !status.success() {
                return Err(anyhow!("7z extraction failed"));
            }
        }
        "rar" => {
            let status = Command::new("unrar")
                .arg("x")
                .arg(archive)
                .arg(dest)
                .status()?;
            if !status.success() {
                return Err(anyhow!("unrar failed"));
            }
        }
        _ => unreachable!(),
    }
    Ok(())
}
