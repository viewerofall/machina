use anyhow::Result;
use notify::{Event, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver};

pub struct FsWatcher {
    watcher: notify::RecommendedWatcher,
    rx: Receiver<notify::Result<Event>>,
    watched: Vec<PathBuf>,
}

impl FsWatcher {
    pub fn new() -> Result<Self> {
        let (tx, rx) = channel();
        let watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
            let _ = tx.send(res);
        })?;
        Ok(Self {
            watcher,
            rx,
            watched: Vec::new(),
        })
    }

    pub fn watch(&mut self, path: &Path) -> Result<()> {
        if self.watched.iter().any(|p| p == path) {
            return Ok(());
        }
        self.watcher.watch(path, RecursiveMode::NonRecursive)?;
        self.watched.push(path.to_path_buf());
        Ok(())
    }

    pub fn unwatch_all(&mut self) {
        for p in self.watched.drain(..) {
            let _ = self.watcher.unwatch(&p);
        }
    }

    /// Drain pending events. Returns true if there was activity worth reloading on.
    pub fn drain(&mut self) -> bool {
        let mut any = false;
        while let Ok(res) = self.rx.try_recv() {
            if let Ok(event) = res {
                if matches!(
                    event.kind,
                    notify::EventKind::Create(_)
                        | notify::EventKind::Remove(_)
                        | notify::EventKind::Modify(notify::event::ModifyKind::Name(_))
                ) {
                    any = true;
                }
            }
        }
        any
    }
}
