use anyhow::Result;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct TrashEntry {
    pub original: PathBuf,
    pub name: String,
    pub deleted: String,
    pub raw_id: String, // internal id from trash crate
}

pub struct TrashView {
    pub items: Vec<TrashEntry>,
    pub cursor: usize,
    pub offset: usize,
    pub selected: std::collections::HashSet<usize>,
}

impl TrashView {
    pub fn open() -> Result<Self> {
        let items = list_items()?;
        Ok(Self {
            items,
            cursor: 0,
            offset: 0,
            selected: Default::default(),
        })
    }

    pub fn refresh(&mut self) -> Result<()> {
        self.items = list_items()?;
        if self.cursor >= self.items.len() {
            self.cursor = self.items.len().saturating_sub(1);
        }
        self.selected.clear();
        Ok(())
    }

    pub fn cursor_down(&mut self, h: u16) {
        if self.cursor + 1 < self.items.len() {
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
        self.cursor = self.items.len().saturating_sub(1);
    }

    pub fn toggle_select(&mut self) {
        if !self.selected.remove(&self.cursor) {
            self.selected.insert(self.cursor);
        }
    }

    pub fn target_indices(&self) -> Vec<usize> {
        if self.selected.is_empty() {
            vec![self.cursor]
        } else {
            let mut v: Vec<_> = self.selected.iter().copied().collect();
            v.sort();
            v
        }
    }

    pub fn restore_selected(&mut self) -> Result<(usize, usize)> {
        let idx = self.target_indices();
        let mut ok = 0;
        let mut err = 0;
        for i in idx.iter().rev() {
            if let Some(item) = self.items.get(*i).cloned() {
                if restore_one(&item).is_ok() {
                    ok += 1;
                } else {
                    err += 1;
                }
            }
        }
        self.refresh()?;
        Ok((ok, err))
    }

    pub fn purge_selected(&mut self) -> Result<(usize, usize)> {
        let idx = self.target_indices();
        let mut ok = 0;
        let mut err = 0;
        for i in idx.iter().rev() {
            if let Some(item) = self.items.get(*i).cloned() {
                if purge_one(&item).is_ok() {
                    ok += 1;
                } else {
                    err += 1;
                }
            }
        }
        self.refresh()?;
        Ok((ok, err))
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
fn list_items() -> Result<Vec<TrashEntry>> {
    use trash::os_limited;
    let mut out = Vec::new();
    for it in os_limited::list()? {
        let name = it
            .original_path()
            .file_name()
            .and_then(|n| n.to_str())
            .map(String::from)
            .unwrap_or_else(|| it.name.to_string_lossy().to_string());
        let deleted = chrono::DateTime::from_timestamp(it.time_deleted, 0)
            .map(|dt| {
                dt.with_timezone(&chrono::Local)
                    .format("%m-%d %H:%M")
                    .to_string()
            })
            .unwrap_or_else(|| "?".to_string());
        out.push(TrashEntry {
            original: it.original_path(),
            name,
            deleted,
            raw_id: it.id.to_string_lossy().to_string(),
        });
    }
    out.sort_by(|a, b| b.deleted.cmp(&a.deleted));
    Ok(out)
}

#[cfg(not(all(unix, not(target_os = "macos"))))]
fn list_items() -> Result<Vec<TrashEntry>> {
    Ok(Vec::new())
}

#[cfg(all(unix, not(target_os = "macos")))]
fn restore_one(entry: &TrashEntry) -> Result<()> {
    use trash::os_limited;
    for it in os_limited::list()? {
        if it.id.to_string_lossy() == entry.raw_id {
            os_limited::restore_all([it])?;
            return Ok(());
        }
    }
    Err(anyhow::anyhow!("trash item gone"))
}
#[cfg(not(all(unix, not(target_os = "macos"))))]
fn restore_one(_entry: &TrashEntry) -> Result<()> {
    Err(anyhow::anyhow!("unsupported platform"))
}

#[cfg(all(unix, not(target_os = "macos")))]
fn purge_one(entry: &TrashEntry) -> Result<()> {
    use trash::os_limited;
    for it in os_limited::list()? {
        if it.id.to_string_lossy() == entry.raw_id {
            os_limited::purge_all([it])?;
            return Ok(());
        }
    }
    Err(anyhow::anyhow!("trash item gone"))
}
#[cfg(not(all(unix, not(target_os = "macos"))))]
fn purge_one(_entry: &TrashEntry) -> Result<()> {
    Err(anyhow::anyhow!("unsupported platform"))
}
