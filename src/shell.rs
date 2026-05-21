use anyhow::Result;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Parsed shell command.
#[derive(Debug, Clone)]
pub struct ShellCmd {
    pub raw: String,
    pub background: bool,
}

/// Parse a shell command. Trailing `&` means background.
pub fn parse(input: &str) -> Option<ShellCmd> {
    let s = input.trim();
    if s.is_empty() {
        return None;
    }
    let (raw, bg) = if let Some(stripped) = s.strip_suffix('&') {
        (stripped.trim().to_string(), true)
    } else {
        (s.to_string(), false)
    };
    if raw.is_empty() {
        return None;
    }
    Some(ShellCmd {
        raw,
        background: bg,
    })
}

/// Expand templates in the command string.
///
/// - `$f` — hovered file (absolute path, shell-quoted)
/// - `$F` — hovered filename only (basename)
/// - `$d` — current directory
/// - `$@` — selected files (space-separated, each quoted); falls back to `$f`
pub fn expand(input: &str, hovered: Option<&Path>, selected: &[PathBuf], cwd: &Path) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '$' {
            out.push(c);
            continue;
        }
        match chars.peek() {
            Some(&'f') => {
                chars.next();
                if let Some(p) = hovered {
                    out.push_str(&shell_quote(&p.display().to_string()));
                }
            }
            Some(&'F') => {
                chars.next();
                if let Some(p) = hovered {
                    let name = p
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("");
                    out.push_str(&shell_quote(name));
                }
            }
            Some(&'d') => {
                chars.next();
                out.push_str(&shell_quote(&cwd.display().to_string()));
            }
            Some(&'@') => {
                chars.next();
                if !selected.is_empty() {
                    let joined: Vec<String> = selected
                        .iter()
                        .map(|p| shell_quote(&p.display().to_string()))
                        .collect();
                    out.push_str(&joined.join(" "));
                } else if let Some(p) = hovered {
                    out.push_str(&shell_quote(&p.display().to_string()));
                }
            }
            _ => {
                out.push('$');
            }
        }
    }
    out
}

/// Minimal single-quoted shell escape.
pub fn shell_quote(s: &str) -> String {
    if !s.is_empty() && s.chars().all(|c| {
        c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | '/' | ':' | ',')
    }) {
        return s.to_string();
    }
    let mut out = String::with_capacity(s.len() + 2);
    out.push('\'');
    for c in s.chars() {
        if c == '\'' {
            out.push_str("'\\''");
        } else {
            out.push(c);
        }
    }
    out.push('\'');
    out
}

/// Run a shell command. Background commands detach completely.
/// Foreground commands run via $SHELL -c and block.
pub fn run(cmd: &ShellCmd, cwd: &Path) -> Result<()> {
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());

    if cmd.background {
        Command::new(&shell)
            .args(["-c", &cmd.raw])
            .current_dir(cwd)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
    } else {
        Command::new(&shell)
            .args(["-c", &cmd.raw])
            .current_dir(cwd)
            .status()?;
    }
    Ok(())
}
