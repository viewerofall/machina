use crate::git::GitStatus;
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: String,
    pub git: Option<crate::git::GitStatus>,
}

pub struct Folder {
    pub path: PathBuf,
    pub files: Vec<FileEntry>,         // filtered, visible
    pub all_files: Vec<FileEntry>,     // unfiltered (after show_hidden)
    pub cursor: usize,
    pub offset: usize,
    pub show_hidden: bool,
    pub filter: String,
    pub git: Option<GitStatus>,
}

impl Folder {
    pub fn new(path: PathBuf) -> Result<Self> {
        let mut folder = Self {
            path,
            files: Vec::new(),
            all_files: Vec::new(),
            cursor: 0,
            offset: 0,
            show_hidden: false,
            filter: String::new(),
            git: None,
        };
        folder.load()?;
        Ok(folder)
    }

    pub fn load_path(&mut self, path: &Path) -> Result<()> {
        self.path = path.to_path_buf();
        self.filter.clear();
        self.load()
    }

    pub fn load(&mut self) -> Result<()> {
        self.all_files.clear();
        self.cursor = 0;
        self.offset = 0;

        if let Ok(entries) = std::fs::read_dir(&self.path) {
            let mut files: Vec<_> = entries
            .filter_map(|e| {
                e.ok().and_then(|entry| {
                    let path = entry.path();
                    let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("?")
                    .to_string();
                    let metadata = entry.metadata().ok()?;
                    let modified = format_time(metadata.modified().ok()?);

                    Some(FileEntry {
                        path,
                        name,
                        is_dir: metadata.is_dir(),
                         size: metadata.len(),
                         modified,
                         git: None, // Will be populated for dirs on demand
                    })
                })
            })
            .collect();

            files.sort_by(|a, b| match (a.is_dir, b.is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                          (false, true) => std::cmp::Ordering::Greater,
                          _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            });

            self.all_files = files;
        }

        // Refresh git status (sync but fast; only on load, not redraw)
        self.git = GitStatus::detect(&self.path);

        self.apply_filter();
        Ok(())
    }

    pub fn apply_filter(&mut self) {
        let show_hidden = self.show_hidden;
        let filter = self.filter.to_lowercase();

        self.files = self
        .all_files
        .iter()
        .filter(|f| {
            if !show_hidden && f.name.starts_with('.') {
                return false;
            }
            if !filter.is_empty() && !f.name.to_lowercase().contains(&filter) {
                return false;
            }
            true
        })
        .cloned()
        .collect();

        if self.cursor >= self.files.len() {
            self.cursor = self.files.len().saturating_sub(1);
        }
        if self.offset > self.cursor {
            self.offset = self.cursor;
        }
    }

    pub fn toggle_hidden(&mut self) {
        self.show_hidden = !self.show_hidden;
        self.apply_filter();
    }

    pub fn set_filter(&mut self, filter: String) {
        self.filter = filter;
        self.cursor = 0;
        self.offset = 0;
        self.apply_filter();
    }

    pub fn clear_filter(&mut self) {
        self.filter.clear();
        self.apply_filter();
    }

    pub fn parent(&mut self) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            let prev = self.path.clone();
            self.path = parent.to_path_buf();
            self.load()?;
            // Position cursor on the directory we just came from
            if let Some(prev_name) = prev.file_name().and_then(|n| n.to_str()) {
                if let Some(idx) = self.files.iter().position(|f| f.name == prev_name) {
                    self.cursor = idx;
                    let viewable = 20;
                    if self.cursor >= viewable {
                        self.offset = self.cursor - viewable + 1;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn enter_dir(&mut self) -> Result<bool> {
        if let Some(entry) = self.files.get(self.cursor) {
            if entry.is_dir {
                self.path = entry.path.clone();
                self.load()?;
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub fn cursor_down(&mut self, h: u16) {
        let h = h.saturating_sub(4) as usize;
        if self.cursor < self.files.len().saturating_sub(1) {
            self.cursor += 1;
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
        self.cursor = self.files.len().saturating_sub(1);
        let viewable = 20;
        if self.cursor >= viewable {
            self.offset = self.cursor - viewable + 1;
        }
    }

    pub fn page_down(&mut self, h: u16) {
        let h = h.saturating_sub(4) as usize;
        self.cursor = (self.cursor + h).min(self.files.len().saturating_sub(1));
        if self.cursor >= self.offset + h {
            self.offset = self.cursor.saturating_sub(h) + 1;
        }
    }

    pub fn page_up(&mut self, h: u16) {
        let h = h.saturating_sub(4) as usize;
        self.cursor = self.cursor.saturating_sub(h);
        if self.cursor < self.offset {
            self.offset = self.cursor;
        }
    }

    pub fn hovered(&self) -> Option<&FileEntry> {
        self.files.get(self.cursor)
    }

    pub fn visible_files(&self, h: u16) -> impl Iterator<Item = &FileEntry> {
        let h = h.saturating_sub(4) as usize;
        let end = (self.offset + h).min(self.files.len());
        self.files[self.offset..end].iter()
    }

    pub fn create_file(&mut self, name: &str) -> Result<()> {
        let new = self.path.join(name);
        std::fs::File::create(&new)?;
        self.load()?;
        if let Some(idx) = self.files.iter().position(|f| f.name == name) {
            self.cursor = idx;
        }
        Ok(())
    }

    pub fn create_dir(&mut self, name: &str) -> Result<()> {
        let new = self.path.join(name);
        std::fs::create_dir_all(&new)?;
        self.load()?;
        if let Some(idx) = self.files.iter().position(|f| f.name == name) {
            self.cursor = idx;
        }
        Ok(())
    }

    /// Jump cursor to next file whose name starts with `c` (case-insensitive).
    /// Wraps around. Returns true if a match was found.
    pub fn jump_to_char(&mut self, c: char) -> bool {
        let needle = c.to_lowercase().next().unwrap_or(c);
        let len = self.files.len();
        if len == 0 {
            return false;
        }
        // search after current cursor, then wrap
        for offset in 1..=len {
            let idx = (self.cursor + offset) % len;
            let first = self.files[idx].name.chars().next();
            if let Some(fc) = first {
                if fc.to_lowercase().next().unwrap_or(fc) == needle {
                    self.cursor = idx;
                    let viewable = 20;
                    if self.cursor < self.offset {
                        self.offset = self.cursor;
                    } else if self.cursor >= self.offset + viewable {
                        self.offset = self.cursor.saturating_sub(viewable - 1);
                    }
                    return true;
                }
            }
        }
        false
    }

    pub fn rename_selected(&mut self, new_name: &str) -> Result<()> {
        if let Some(entry) = self.files.get(self.cursor).cloned() {
            let new = self.path.join(new_name);
            std::fs::rename(&entry.path, &new)?;
            self.load()?;
            if let Some(idx) = self.files.iter().position(|f| f.name == new_name) {
                self.cursor = idx;
            }
        }
        Ok(())
    }
}

fn format_time(time: SystemTime) -> String {
    use std::time::UNIX_EPOCH;
    let duration = time.duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = duration.as_secs() as i64;

    if let Some(dt) = chrono::DateTime::from_timestamp(secs, 0) {
        let local = dt.with_timezone(&chrono::Local);
        local.format("%m-%d %H:%M").to_string()
    } else {
        "?".to_string()
    }
}

#[allow(dead_code)]
pub fn name_of(p: &Path) -> &str {
    p.file_name().and_then(|n| n.to_str()).unwrap_or("?")
}
