use crate::git::GitStatus;
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortMode {
    Name,
    Size,
    Modified,
    Extension,
}

impl SortMode {
    pub fn label(self) -> &'static str {
        match self {
            SortMode::Name => "name",
            SortMode::Size => "size",
            SortMode::Modified => "mtime",
            SortMode::Extension => "ext",
        }
    }
}

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub mtime_secs: i64,
    pub modified: String,
    pub git: Option<crate::git::GitStatus>,
    pub computed_size: Option<u64>,
    pub is_symlink: bool,
    pub symlink_target: Option<PathBuf>,
    pub perms: String,
    pub is_ignored: bool,
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
    pub sort: SortMode,
    pub sort_reverse: bool,
    pub respect_gitignore: bool,
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
            sort: SortMode::Name,
            sort_reverse: false,
            respect_gitignore: true,
        };
        folder.load()?;
        Ok(folder)
    }

    pub fn set_sort(&mut self, sort: SortMode) {
        if self.sort == sort {
            self.sort_reverse = !self.sort_reverse;
        } else {
            self.sort = sort;
            self.sort_reverse = false;
        }
        self.sort_files();
        self.apply_filter();
    }

    pub fn sort_files(&mut self) {
        let rev = self.sort_reverse;
        let mode = self.sort;
        self.all_files.sort_by(|a, b| {
            // dirs first regardless
            let dir_cmp = match (a.is_dir, b.is_dir) {
                (true, false) => return std::cmp::Ordering::Less,
                (false, true) => return std::cmp::Ordering::Greater,
                _ => std::cmp::Ordering::Equal,
            };
            let _ = dir_cmp;
            let ord = match mode {
                SortMode::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                SortMode::Size => {
                    let sa = a.computed_size.unwrap_or(a.size);
                    let sb = b.computed_size.unwrap_or(b.size);
                    sb.cmp(&sa) // biggest first
                }
                SortMode::Modified => b.mtime_secs.cmp(&a.mtime_secs), // newest first
                SortMode::Extension => {
                    let ea = a.path.extension().and_then(|e| e.to_str()).unwrap_or("");
                    let eb = b.path.extension().and_then(|e| e.to_str()).unwrap_or("");
                    ea.to_lowercase().cmp(&eb.to_lowercase())
                        .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
                }
            };
            if rev { ord.reverse() } else { ord }
        });
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

        // Build gitignore matcher (None if no .gitignore upward)
        let ignore_set: std::collections::HashSet<PathBuf> = if self.respect_gitignore {
            build_ignore_set(&self.path)
        } else {
            Default::default()
        };

        if let Ok(entries) = std::fs::read_dir(&self.path) {
            let files: Vec<_> = entries
            .filter_map(|e| {
                e.ok().and_then(|entry| {
                    let path = entry.path();
                    let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("?")
                    .to_string();
                    // Use symlink_metadata so we don't follow links
                    let lmeta = entry.metadata().ok()?;
                    let ft = lmeta.file_type();
                    let is_symlink = ft.is_symlink();
                    let symlink_target = if is_symlink {
                        std::fs::read_link(&path).ok()
                    } else {
                        None
                    };
                    // For dir detection, follow link
                    let is_dir = if is_symlink {
                        std::fs::metadata(&path).map(|m| m.is_dir()).unwrap_or(false)
                    } else {
                        lmeta.is_dir()
                    };

                    let mtime = lmeta.modified().ok()?;
                    let modified = format_time(mtime);
                    let mtime_secs = mtime
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_secs() as i64)
                        .unwrap_or(0);
                    let computed_size = if is_dir {
                        crate::dir_size::lookup(&path)
                    } else {
                        None
                    };
                    let perms = format_perms(&lmeta);
                    let is_ignored = ignore_set.contains(&path);

                    Some(FileEntry {
                        path,
                        name,
                        is_dir,
                        size: lmeta.len(),
                        mtime_secs,
                        modified,
                        git: None,
                        computed_size,
                        is_symlink,
                        symlink_target,
                        perms,
                        is_ignored,
                    })
                })
            })
            .collect();

            self.all_files = files;
            self.sort_files();
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
            if !show_hidden && (f.name.starts_with('.') || f.is_ignored) {
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

    pub fn rename_selected(&mut self, new_name: &str) -> Result<Option<(PathBuf, PathBuf)>> {
        let mut moved: Option<(PathBuf, PathBuf)> = None;
        if let Some(entry) = self.files.get(self.cursor).cloned() {
            let new = self.path.join(new_name);
            std::fs::rename(&entry.path, &new)?;
            moved = Some((entry.path.clone(), new.clone()));
            self.load()?;
            if let Some(idx) = self.files.iter().position(|f| f.name == new_name) {
                self.cursor = idx;
            }
        }
        Ok(moved)
    }
}

fn format_perms(meta: &std::fs::Metadata) -> String {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = meta.permissions().mode();
        let ft = meta.file_type();
        let kind = if ft.is_symlink() {
            'l'
        } else if ft.is_dir() {
            'd'
        } else {
            '-'
        };
        let bit = |n: u32, ch: char| if mode & (1 << n) != 0 { ch } else { '-' };
        format!(
            "{}{}{}{}{}{}{}{}{}{}",
            kind,
            bit(8, 'r'),
            bit(7, 'w'),
            bit(6, 'x'),
            bit(5, 'r'),
            bit(4, 'w'),
            bit(3, 'x'),
            bit(2, 'r'),
            bit(1, 'w'),
            bit(0, 'x'),
        )
    }
    #[cfg(not(unix))]
    {
        let _ = meta;
        "----------".to_string()
    }
}

fn build_ignore_set(path: &Path) -> std::collections::HashSet<PathBuf> {
    // Walk just this dir (depth=1) using ignore's WalkBuilder so we apply
    // .gitignore + global ignores. Anything *not* visited is ignored.
    use std::collections::HashSet;
    let mut visited: HashSet<PathBuf> = HashSet::new();
    let walk = ignore::WalkBuilder::new(path)
        .max_depth(Some(1))
        .hidden(false)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .require_git(false)
        .build();
    for e in walk.flatten() {
        if e.path() == path {
            continue;
        }
        visited.insert(e.path().to_path_buf());
    }
    // Anything in std::fs::read_dir but not in visited is ignored.
    let mut ignored = HashSet::new();
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if !visited.contains(&p) {
                ignored.insert(p);
            }
        }
    }
    ignored
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
