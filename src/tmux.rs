use anyhow::Result;
use std::path::Path;
use std::process::{Command, Stdio};

/// True if running inside a tmux session.
pub fn in_tmux() -> bool {
    std::env::var_os("TMUX").is_some()
}

/// Open a horizontal split-window in tmux running the given command in `cwd`.
/// `cmd_args` is the program to spawn (e.g. ["nvim", "/tmp/file"] or just ["zsh"]).
pub fn split(cwd: &Path, cmd_args: &[&str], horizontal: bool) -> Result<()> {
    let mut cmd = Command::new("tmux");
    cmd.arg("split-window");
    if horizontal {
        cmd.arg("-h");
    }
    cmd.arg("-c").arg(cwd);
    for a in cmd_args {
        cmd.arg(a);
    }
    cmd.stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;
    Ok(())
}

/// Open a new tmux window (in current session) running cmd in cwd.
pub fn new_window(cwd: &Path, cmd_args: &[&str]) -> Result<()> {
    let mut cmd = Command::new("tmux");
    cmd.arg("new-window").arg("-c").arg(cwd);
    for a in cmd_args {
        cmd.arg(a);
    }
    cmd.stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;
    Ok(())
}
