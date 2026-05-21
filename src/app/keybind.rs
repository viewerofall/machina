use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Visual,
}

pub struct KeybindState {
    pub mode: Mode,
    pub pending_key: Option<(char, Instant)>,
    pub visual_start: Option<usize>,
    pub visual_end: Option<usize>,
    pub search_mode: bool,
}

impl Default for KeybindState {
    fn default() -> Self {
        Self {
            mode: Mode::Normal,
            pending_key: None,
            visual_start: None,
            visual_end: None,
            search_mode: false,
        }
    }
}

impl KeybindState {
    pub fn clear_pending(&mut self) {
        self.pending_key = None;
    }

    pub fn set_pending(&mut self, key: char) {
        self.pending_key = Some((key, Instant::now()));
    }

    pub fn get_pending(&self) -> Option<char> {
        self.pending_key.and_then(|(k, time)| {
            // Clear pending after 1 second
            if time.elapsed() > Duration::from_secs(1) {
                None
            } else {
                Some(k)
            }
        })
    }

    pub fn enter_visual(&mut self, cursor: usize) {
        self.mode = Mode::Visual;
        self.visual_start = Some(cursor);
        self.visual_end = Some(cursor);
    }

    pub fn exit_visual(&mut self) {
        self.mode = Mode::Normal;
        self.visual_start = None;
        self.visual_end = None;
    }

    pub fn extend_visual(&mut self, cursor: usize) {
        if self.mode == Mode::Visual {
            self.visual_end = Some(cursor);
        }
    }

    pub fn get_visual_range(&self) -> Option<(usize, usize)> {
        match (self.visual_start, self.visual_end) {
            (Some(start), Some(end)) => {
                if start <= end {
                    Some((start, end))
                } else {
                    Some((end, start))
                }
            }
            _ => None,
        }
    }
}
