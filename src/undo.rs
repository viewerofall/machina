use anyhow::{anyhow, Result};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum UndoOp {
    /// File was moved/renamed from src to dst — undo restores it
    Move { src: PathBuf, dst: PathBuf },
    /// File was created by user — undo deletes it
    Created { path: PathBuf },
    /// File was trashed — undo isn't possible from here (use T)
    Trashed { paths: Vec<PathBuf> },
}

#[derive(Default)]
pub struct UndoStack {
    pub ops: Vec<UndoOp>,
}

impl UndoStack {
    pub fn push(&mut self, op: UndoOp) {
        self.ops.push(op);
        if self.ops.len() > 32 {
            self.ops.remove(0);
        }
    }

    pub fn pop_and_apply(&mut self) -> Result<String> {
        let op = self.ops.pop().ok_or_else(|| anyhow!("nothing to undo"))?;
        match op {
            UndoOp::Move { src, dst } => {
                std::fs::rename(&dst, &src)?;
                Ok(format!(
                    "undo move: {} ← {}",
                    src.file_name().and_then(|n| n.to_str()).unwrap_or("?"),
                    dst.display()
                ))
            }
            UndoOp::Created { path } => {
                if path.is_dir() {
                    std::fs::remove_dir(&path)?;
                } else {
                    std::fs::remove_file(&path)?;
                }
                Ok(format!("undo create: removed {}", path.display()))
            }
            UndoOp::Trashed { paths: _ } => {
                Err(anyhow!("trash undo: open trash browser (T) to restore"))
            }
        }
    }
}
