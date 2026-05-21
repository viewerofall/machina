use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Debug, Clone)]
pub struct GitStatus {
    pub branch: String,
    pub clean: bool,
    pub modified: usize,
    pub untracked: usize,
    pub staged: usize,
    pub ahead: u32,
    pub behind: u32,
}

impl GitStatus {
    /// Quick check: are we in a git repo at all?
    pub fn detect(path: &Path) -> Option<Self> {
        let repo_root = find_git_root(path)?;
        run_status(&repo_root)
    }

    pub fn is_dirty(&self) -> bool {
        !self.clean
    }
}

fn find_git_root(start: &Path) -> Option<PathBuf> {
    let mut cur = start.to_path_buf();
    loop {
        if cur.join(".git").exists() {
            return Some(cur);
        }
        if !cur.pop() {
            return None;
        }
    }
}

fn run_status(repo: &Path) -> Option<GitStatus> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["status", "--porcelain=v1", "--branch"])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let text = String::from_utf8_lossy(&output.stdout);
    parse_porcelain(&text)
}

fn parse_porcelain(text: &str) -> Option<GitStatus> {
    let mut branch = String::from("?");
    let mut clean = true;
    let mut modified = 0usize;
    let mut untracked = 0usize;
    let mut staged = 0usize;
    let mut ahead = 0u32;
    let mut behind = 0u32;

    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("## ") {
            // Examples:
            //   "## main...origin/main"
            //   "## main...origin/main [ahead 1]"
            //   "## main...origin/main [ahead 1, behind 2]"
            //   "## HEAD (no branch)"
            let head = rest.split("...").next().unwrap_or(rest);
            branch = head.split_whitespace().next().unwrap_or("?").to_string();

            if let Some(start) = rest.find('[') {
                if let Some(end) = rest[start..].find(']') {
                    let inside = &rest[start + 1..start + end];
                    for token in inside.split(',') {
                        let t = token.trim();
                        if let Some(n) = t.strip_prefix("ahead ") {
                            ahead = n.parse().unwrap_or(0);
                        } else if let Some(n) = t.strip_prefix("behind ") {
                            behind = n.parse().unwrap_or(0);
                        }
                    }
                }
            }
            continue;
        }
        if line.is_empty() {
            continue;
        }

        clean = false;
        let bytes = line.as_bytes();
        if bytes.len() < 2 {
            continue;
        }
        let xy = (bytes[0] as char, bytes[1] as char);
        match xy {
            ('?', '?') => untracked += 1,
            (x, y) => {
                if x != ' ' && x != '?' {
                    staged += 1;
                }
                if y != ' ' && y != '?' {
                    modified += 1;
                }
            }
        }
    }

    Some(GitStatus {
        branch,
        clean,
        modified,
        untracked,
        staged,
        ahead,
        behind,
    })
}
