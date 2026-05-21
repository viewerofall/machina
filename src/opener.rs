use crate::config::Config;
use anyhow::Result;
use std::path::Path;
use std::process::{Command, Stdio};

/// Open a file with the best handler. Looks up the extension in config.openers.
/// Falls back to xdg-open.
pub fn open(path: &Path, config: &Config) -> Result<()> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_lowercase);

    let cmd = ext
        .as_deref()
        .and_then(|e| config.openers.get(e).cloned())
        .unwrap_or_else(|| "xdg-open".to_string());

    let block = is_terminal_app(&cmd);

    if block {
        // Crossterm raw mode is on - we need to spawn into the existing terminal
        // and the editor will take over. Caller is responsible for suspending the
        // alternate screen.
        Command::new(&cmd).arg(path).status()?;
    } else {
        Command::new(&cmd)
            .arg(path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
    }

    Ok(())
}

pub fn open_with(command: &str, path: &Path) -> Result<()> {
    let block = is_terminal_app(command);
    if block {
        Command::new(command).arg(path).status()?;
    } else {
        Command::new(command)
            .arg(path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
    }
    Ok(())
}

pub fn is_terminal_app(cmd: &str) -> bool {
    let base = std::path::Path::new(cmd)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(cmd);
    matches!(
        base,
        "nvim" | "vim" | "vi" | "emacs" | "nano" | "less" | "more" | "lvim" | "hx" | "helix"
    )
}

pub fn open_shell(cwd: &Path) -> Result<()> {
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
    Command::new(shell).current_dir(cwd).status()?;
    Ok(())
}
