use crate::app::{App, Confirm, ConfirmKind, FileOp, OpMode, PasteDialog};
use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

pub fn yank_targets(app: &mut App) -> Result<()> {
    let targets = app.targets();
    if targets.is_empty() {
        return Ok(());
    }
    let n = targets.len();
    app.file_op = Some(FileOp {
        mode: OpMode::Copy,
        files: targets,
    });
    app.message(format!("yanked {} item{}", n, plural(n)));
    Ok(())
}

pub fn cut_targets(app: &mut App) -> Result<()> {
    let targets = app.targets();
    if targets.is_empty() {
        return Ok(());
    }
    let n = targets.len();
    app.file_op = Some(FileOp {
        mode: OpMode::Cut,
        files: targets,
    });
    app.message(format!("cut {} item{}", n, plural(n)));
    Ok(())
}

/// Open the paste dialog (Copy/Move/Link). Defaults to current op mode.
pub fn open_paste_dialog(app: &mut App) {
    let default = match app.file_op.as_ref().map(|o| o.mode) {
        Some(OpMode::Copy) => 0,
        Some(OpMode::Cut) => 1,
        Some(OpMode::Link) => 2,
        None => {
            app.message("clipboard empty".to_string());
            return;
        }
    };
    app.paste_dialog = Some(PasteDialog {
        selected: default,
        default,
        dest_override: None,
    });
}

/// Send selected/hovered files to the *other* pane (in split mode).
/// Yanks them as Copy, opens the paste dialog targeting the other pane.
pub fn send_to_other_pane(app: &mut App) {
    let Some(other) = app.split else {
        app.message("not in split mode (press `|`)".to_string());
        return;
    };
    let targets = app.targets();
    if targets.is_empty() {
        return;
    }
    let dest = app.tabs[other].path.clone();
    let n = targets.len();
    app.file_op = Some(FileOp {
        mode: OpMode::Copy,
        files: targets,
    });
    app.paste_dialog = Some(PasteDialog {
        selected: 0, // Copy default
        default: 0,
        dest_override: Some(dest),
    });
    app.message(format!("send {} item(s) to other pane", n));
}

/// Execute paste with chosen mode.
pub fn commit_paste(app: &mut App, mode: OpMode, dest_override: Option<PathBuf>) -> Result<()> {
    let Some(op) = app.file_op.clone() else {
        app.message("clipboard empty".to_string());
        return Ok(());
    };

    let dest_dir = dest_override.unwrap_or_else(|| app.current().path.clone());
    let mut ok = 0;
    let mut skipped = 0;

    for src in &op.files {
        let name = match src.file_name() {
            Some(n) => n,
            None => continue,
        };
        let dest = unique_dest(&dest_dir, name);

        let result = match mode {
            OpMode::Copy => {
                if src.is_dir() {
                    copy_dir_all(src, &dest)
                } else {
                    fs::copy(src, &dest).map(|_| ()).map_err(Into::into)
                }
            }
            OpMode::Cut => fs::rename(src, &dest).map_err(Into::into),
            OpMode::Link => {
                #[cfg(unix)]
                {
                    std::os::unix::fs::symlink(src, &dest).map_err(Into::into)
                }
                #[cfg(not(unix))]
                {
                    Err(anyhow::anyhow!("symlinks unsupported on this platform"))
                }
            }
        };

        match result {
            Ok(()) => ok += 1,
            Err(_) => skipped += 1,
        }
    }

    let label = match mode {
        OpMode::Copy => "copied",
        OpMode::Cut => "moved",
        OpMode::Link => "linked",
    };
    if skipped > 0 {
        app.message(format!("{} {} ({} skipped)", label, ok, skipped));
    } else {
        app.message(format!("{} {}", label, ok));
    }

    // Yank persists; cut/link consume
    if mode == OpMode::Cut || mode == OpMode::Link {
        app.file_op = None;
    }
    app.clear_selected();
    // Reload all tabs so destination shows new files
    for t in app.tabs.iter_mut() {
        let _ = t.load();
    }
    Ok(())
}

pub fn trash_targets(app: &mut App) -> Result<()> {
    let targets = app.targets();
    if targets.is_empty() {
        return Ok(());
    }
    if !app.config.confirm_delete {
        return do_trash(app, targets);
    }
    let msg = match targets.len() {
        1 => format!(
            "Move to trash: {}? (y/N)",
            targets[0].file_name().and_then(|n| n.to_str()).unwrap_or("?")
        ),
        n => format!("Move {} item{} to trash? (y/N)", n, plural(n)),
    };
    app.confirm = Some(Confirm {
        kind: ConfirmKind::DeleteTrash,
        message: msg,
        targets,
    });
    Ok(())
}

pub fn delete_targets_permanent(app: &mut App) -> Result<()> {
    let targets = app.targets();
    if targets.is_empty() {
        return Ok(());
    }
    let msg = match targets.len() {
        1 => format!(
            "PERMANENTLY delete: {}? (y/N)",
            targets[0].file_name().and_then(|n| n.to_str()).unwrap_or("?")
        ),
        n => format!("PERMANENTLY delete {} item{}? (y/N)", n, plural(n)),
    };
    app.confirm = Some(Confirm {
        kind: ConfirmKind::DeletePermanent,
        message: msg,
        targets,
    });
    Ok(())
}

pub fn confirm_yes(app: &mut App) -> Result<()> {
    let Some(c) = app.confirm.take() else {
        return Ok(());
    };
    match c.kind {
        ConfirmKind::DeleteTrash => do_trash(app, c.targets)?,
        ConfirmKind::DeletePermanent => {
            let mut ok = 0;
            let mut skipped = 0;
            for p in c.targets {
                let res = if p.is_dir() {
                    fs::remove_dir_all(&p)
                } else {
                    fs::remove_file(&p)
                };
                if res.is_ok() {
                    ok += 1;
                } else {
                    skipped += 1;
                }
            }
            if skipped > 0 {
                app.message(format!("deleted {} ({} skipped)", ok, skipped));
            } else {
                app.message(format!("deleted {}", ok));
            }
            app.clear_selected();
            app.current_mut().load()?;
        }
        ConfirmKind::Archive { name, cwd } => {
            match crate::archive::create(&name, &cwd, &c.targets) {
                Ok(()) => app.message(format!("archived → {}", name)),
                Err(e) => app.message(format!("archive failed: {}", e)),
            }
            app.clear_selected();
            app.current_mut().load()?;
        }
    }
    Ok(())
}

pub fn confirm_no(app: &mut App) {
    app.confirm = None;
    app.message("cancelled".to_string());
}

fn do_trash(app: &mut App, targets: Vec<PathBuf>) -> Result<()> {
    let mut ok = 0;
    let mut skipped = 0;
    for p in targets {
        if trash::delete(&p).is_ok() {
            ok += 1;
        } else {
            skipped += 1;
        }
    }
    if skipped > 0 {
        app.message(format!("trashed {} ({} skipped)", ok, skipped));
    } else {
        app.message(format!("trashed {}", ok));
    }
    app.clear_selected();
    app.current_mut().load()?;
    Ok(())
}

fn unique_dest(dir: &Path, name: &std::ffi::OsStr) -> PathBuf {
    let candidate = dir.join(name);
    if !candidate.exists() {
        return candidate;
    }

    let name_str = name.to_string_lossy();
    let (stem, ext) = match name_str.rfind('.') {
        Some(i) if i > 0 => (&name_str[..i], &name_str[i..]),
        _ => (name_str.as_ref(), ""),
    };

    for n in 1..1000 {
        let new = dir.join(format!("{} ({}){}", stem, n, ext));
        if !new.exists() {
            return new;
        }
    }
    candidate
}

fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let path = entry.path();
        let name = entry.file_name();
        let dest = dst.join(&name);
        if ty.is_dir() {
            copy_dir_all(&path, &dest)?;
        } else {
            fs::copy(&path, &dest)?;
        }
    }
    Ok(())
}

fn plural(n: usize) -> &'static str {
    if n == 1 {
        ""
    } else {
        "s"
    }
}
