use anyhow::{anyhow, Result};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Open paths in $EDITOR; on save, rename per line. Returns (renamed, skipped).
pub fn run(editor: &str, paths: &[PathBuf]) -> Result<(usize, usize)> {
    if paths.is_empty() {
        return Ok((0, 0));
    }
    let dir = paths[0]
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));

    let tmp = std::env::temp_dir().join(format!("machina-bulk-{}.txt", std::process::id()));
    {
        let mut f = std::fs::File::create(&tmp)?;
        writeln!(
            f,
            "# bulk rename: edit names below, save & quit. Lines must match count."
        )?;
        writeln!(f, "# parent: {}", dir.display())?;
        for p in paths {
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            writeln!(f, "{}", name)?;
        }
    }

    let status = Command::new(editor).arg(&tmp).status()?;
    if !status.success() {
        let _ = std::fs::remove_file(&tmp);
        return Err(anyhow!("editor exited with error"));
    }

    let body = std::fs::read_to_string(&tmp)?;
    let _ = std::fs::remove_file(&tmp);

    let new_names: Vec<&str> = body
        .lines()
        .filter(|l| !l.trim_start().starts_with('#') && !l.trim().is_empty())
        .collect();

    if new_names.len() != paths.len() {
        return Err(anyhow!(
            "line count mismatch: expected {}, got {}",
            paths.len(),
            new_names.len()
        ));
    }

    let mut ok = 0;
    let mut skipped = 0;
    for (src, new_name) in paths.iter().zip(new_names.iter()) {
        let old_name = src.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if old_name == *new_name {
            continue;
        }
        if new_name.contains('/') {
            skipped += 1;
            continue;
        }
        let dest = src.with_file_name(new_name);
        if dest.exists() {
            skipped += 1;
            continue;
        }
        if std::fs::rename(src, &dest).is_ok() {
            ok += 1;
        } else {
            skipped += 1;
        }
    }
    Ok((ok, skipped))
}
