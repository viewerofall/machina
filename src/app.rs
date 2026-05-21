pub mod folder;
pub mod keybind;

use crate::config::Config;
use crate::input::Input;
use anyhow::Result;
use std::collections::{HashSet, VecDeque};
use std::path::PathBuf;

pub use folder::{Folder, SortMode};
pub use keybind::{KeybindState, Mode};

#[derive(Debug, Clone)]
pub struct FileOp {
    pub mode: OpMode,
    pub files: Vec<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OpMode {
    Copy,
    Cut,
    Link,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConfirmKind {
    DeleteTrash,
    DeletePermanent,
    Archive { name: String, cwd: PathBuf },
}

#[derive(Debug, Clone)]
pub struct Confirm {
    pub kind: ConfirmKind,
    pub message: String,
    pub targets: Vec<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct PasteDialog {
    pub selected: usize, // 0=copy, 1=move, 2=link
    pub default: usize,
    pub dest_override: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HelpVisible {
    Hidden,
    Shown,
}

pub struct App {
    pub tabs: Vec<Folder>,
    pub active: usize,
    pub split: Option<usize>,         // if Some(i), tabs[i] visible on right pane
    pub selected: HashSet<PathBuf>,   // persistent multi-select
    pub keybind: KeybindState,
    pub file_op: Option<FileOp>,
    pub messages: VecDeque<String>,
    pub preview_visible: bool,
    pub input: Input,
    pub confirm: Option<Confirm>,
    pub paste_dialog: Option<PasteDialog>,
    pub help: HelpVisible,
    pub config: Config,
    pub pending_fg_shell: Option<crate::shell::ShellCmd>,
    pub trash_view: Option<crate::trash_view::TrashView>,
    pub du_view: Option<crate::du_view::DuView>,
    pub undo_stack: crate::undo::UndoStack,
}

impl App {
    pub fn new(start_path: Option<PathBuf>, config: Config) -> Result<Self> {
        let path = start_path
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/")));

        let mut folder = Folder::new(path)?;
        folder.show_hidden = config.show_hidden;
        folder.respect_gitignore = config.respect_gitignore;
        // Re-load now that the ignore flag is set, so files get tagged correctly.
        let _ = folder.load();
        folder.apply_filter();

        Ok(Self {
            tabs: vec![folder],
            active: 0,
            split: None,
            selected: HashSet::new(),
            keybind: KeybindState::default(),
            file_op: None,
            messages: VecDeque::new(),
            preview_visible: true,
            input: Input::default(),
            confirm: None,
            paste_dialog: None,
            help: HelpVisible::Hidden,
            config,
            pending_fg_shell: None,
            trash_view: None,
            du_view: None,
            undo_stack: Default::default(),
        })
    }

    pub fn current(&self) -> &Folder {
        &self.tabs[self.active]
    }

    pub fn current_mut(&mut self) -> &mut Folder {
        &mut self.tabs[self.active]
    }

    pub fn message(&mut self, msg: String) {
        self.messages.push_back(msg);
        if self.messages.len() > 5 {
            self.messages.pop_front();
        }
    }

    pub fn new_tab(&mut self, path: PathBuf) -> Result<()> {
        let mut f = Folder::new(path)?;
        f.show_hidden = self.config.show_hidden;
        f.respect_gitignore = self.config.respect_gitignore;
        let _ = f.load();
        f.apply_filter();
        self.tabs.push(f);
        self.active = self.tabs.len() - 1;
        Ok(())
    }

    pub fn close_tab(&mut self) {
        if self.tabs.len() <= 1 {
            return;
        }
        let removed = self.active;
        self.tabs.remove(removed);
        // Fix split index if affected
        if let Some(s) = self.split {
            if s == removed {
                self.split = None;
            } else if s > removed {
                self.split = Some(s - 1);
            }
        }
        if self.active >= self.tabs.len() {
            self.active = self.tabs.len() - 1;
        }
    }

    pub fn next_tab(&mut self) {
        if self.tabs.is_empty() {
            return;
        }
        self.active = (self.active + 1) % self.tabs.len();
    }

    pub fn prev_tab(&mut self) {
        if self.tabs.is_empty() {
            return;
        }
        self.active = if self.active == 0 {
            self.tabs.len() - 1
        } else {
            self.active - 1
        };
    }

    /// Toggle split-view: if not split, open the current tab path in a second tab
    /// and show side-by-side. If split, close split.
    pub fn toggle_split(&mut self) -> Result<()> {
        if self.split.is_some() {
            self.split = None;
            return Ok(());
        }
        let path = self.current().path.clone();
        self.new_tab(path)?;
        // active is now the new tab; split shows the previous tab
        self.split = Some(self.active - 1);
        Ok(())
    }

    /// Swap which side has focus in split mode
    pub fn swap_split_focus(&mut self) {
        if let Some(s) = self.split {
            let was = self.active;
            self.active = s;
            self.split = Some(was);
        }
    }

    // Persistent multi-select
    pub fn toggle_selected(&mut self, path: PathBuf) {
        if !self.selected.remove(&path) {
            self.selected.insert(path);
        }
    }

    pub fn clear_selected(&mut self) {
        self.selected.clear();
    }

    pub fn select_all(&mut self) {
        for entry in self.current().files.clone() {
            self.selected.insert(entry.path);
        }
    }

    /// Files to operate on: selected set if any, else hovered.
    pub fn targets(&self) -> Vec<PathBuf> {
        if !self.selected.is_empty() {
            self.selected.iter().cloned().collect()
        } else if let Some(e) = self.current().hovered() {
            vec![e.path.clone()]
        } else {
            vec![]
        }
    }
}
