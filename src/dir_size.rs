use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Entry {
    mtime_secs: u64,
    bytes: u64,
}

#[derive(Default, Serialize, Deserialize)]
struct CacheFile {
    entries: HashMap<String, Entry>,
}

static CACHE: OnceLock<Mutex<CacheFile>> = OnceLock::new();

fn cache_path() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("machina")
        .join("sizes.json")
}

fn cache() -> &'static Mutex<CacheFile> {
    CACHE.get_or_init(|| {
        let path = cache_path();
        let loaded = std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str::<CacheFile>(&s).ok())
            .unwrap_or_default();
        Mutex::new(loaded)
    })
}

fn save() {
    let path = cache_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(guard) = cache().lock() {
        if let Ok(s) = serde_json::to_string(&*guard) {
            let _ = std::fs::write(&path, s);
        }
    }
}

fn mtime_secs(p: &Path) -> u64 {
    std::fs::metadata(p)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Return cached size if fresh, else None.
pub fn lookup(path: &Path) -> Option<u64> {
    let key = path.to_string_lossy().into_owned();
    let mtime = mtime_secs(path);
    let guard = cache().lock().ok()?;
    let e = guard.entries.get(&key)?;
    if e.mtime_secs == mtime {
        Some(e.bytes)
    } else {
        None
    }
}

/// Compute size of a directory using jwalk (parallel). Files return their size directly.
pub fn compute(path: &Path) -> u64 {
    if path.is_file() {
        return std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    }
    let total: u64 = jwalk::WalkDir::new(path)
        .skip_hidden(false)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter_map(|e| e.metadata().ok())
        .filter(|m| m.is_file())
        .map(|m| m.len())
        .sum();
    total
}

/// Compute & cache. Returns the byte count.
pub fn compute_and_store(path: &Path) -> u64 {
    let bytes = compute(path);
    let mtime = mtime_secs(path);
    if let Ok(mut guard) = cache().lock() {
        guard.entries.insert(
            path.to_string_lossy().into_owned(),
            Entry { mtime_secs: mtime, bytes },
        );
    }
    save();
    bytes
}
