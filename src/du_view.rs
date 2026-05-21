//! Disk usage view — ncdu-style bar chart of items in current dir.

use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct DuEntry {
    pub path: PathBuf,
    pub name: String,
    pub size: u64,
    pub is_dir: bool,
}

pub struct DuView {
    pub entries: Vec<DuEntry>,
    pub cursor: usize,
    pub offset: usize,
    pub total: u64,
    pub dir: PathBuf,
}

impl DuView {
    /// Compute disk usage for all immediate children of `dir`. Uses jwalk for dirs
    /// and the file size cache where available.
    pub fn compute(dir: &std::path::Path) -> Self {
        let mut entries = Vec::new();
        let mut total = 0u64;
        if let Ok(rd) = std::fs::read_dir(dir) {
            for ent in rd.flatten() {
                let p = ent.path();
                let name = p
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("?")
                    .to_string();
                let meta = ent.metadata().ok();
                let is_dir = meta.as_ref().map(|m| m.is_dir()).unwrap_or(false);
                let size = if is_dir {
                    crate::dir_size::lookup(&p)
                        .unwrap_or_else(|| crate::dir_size::compute_and_store(&p))
                } else {
                    meta.map(|m| m.len()).unwrap_or(0)
                };
                total += size;
                entries.push(DuEntry { path: p, name, size, is_dir });
            }
        }
        entries.sort_by(|a, b| b.size.cmp(&a.size));
        Self {
            entries,
            cursor: 0,
            offset: 0,
            total,
            dir: dir.to_path_buf(),
        }
    }

    pub fn cursor_down(&mut self, h: u16) {
        if self.cursor + 1 < self.entries.len() {
            self.cursor += 1;
            let h = h.saturating_sub(4) as usize;
            if self.cursor >= self.offset + h {
                self.offset = self.cursor.saturating_sub(h) + 1;
            }
        }
    }
    pub fn cursor_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            if self.cursor < self.offset {
                self.offset = self.cursor;
            }
        }
    }
    pub fn cursor_top(&mut self) {
        self.cursor = 0;
        self.offset = 0;
    }
    pub fn cursor_bottom(&mut self) {
        self.cursor = self.entries.len().saturating_sub(1);
    }

    pub fn hovered(&self) -> Option<&DuEntry> {
        self.entries.get(self.cursor)
    }
}
