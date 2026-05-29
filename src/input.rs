#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputAction {
    Rename,
    Create,      // smart: f=name / d=name / name
    Search,
    JumpToChar,  // one-char jump (vim 'f')
    Archive,     // create .tar.gz
    Shell,       // run shell command ($f, $@, $d, trailing & = bg)
    Teleport,    // cd to path
    Chmod,       // chmod hovered/selected (octal or symbolic)
    ExtractHere, // extract archive to current directory
    ExtractTo,   // extract archive to specified path
    Extract,     // menu for extract options
}

pub struct Input {
    pub active: bool,
    pub action: InputAction,
    pub buffer: String,
    pub prompt: String,
    pub cursor: usize,
}

impl Default for Input {
    fn default() -> Self {
        Self {
            active: false,
            action: InputAction::Search,
            buffer: String::new(),
            prompt: String::new(),
            cursor: 0,
        }
    }
}

impl Input {
    pub fn open(&mut self, action: InputAction, prompt: String, prefill: String) {
        self.active = true;
        self.action = action;
        self.prompt = prompt;
        self.cursor = prefill.chars().count();
        self.buffer = prefill;
    }

    /// Like open(), but positions cursor right before the extension (yazi `A` behavior).
    pub fn open_before_ext(&mut self, action: InputAction, prompt: String, prefill: String) {
        let cursor = match prefill.rfind('.') {
            Some(i) if i > 0 => prefill[..i].chars().count(),
            _ => prefill.chars().count(),
        };
        self.active = true;
        self.action = action;
        self.prompt = prompt;
        self.buffer = prefill;
        self.cursor = cursor;
    }

    pub fn close(&mut self) {
        self.active = false;
        self.buffer.clear();
        self.prompt.clear();
        self.cursor = 0;
    }

    pub fn insert(&mut self, c: char) {
        let byte_pos = self.byte_pos();
        self.buffer.insert(byte_pos, c);
        self.cursor += 1;
    }

    pub fn backspace(&mut self) {
        if self.cursor == 0 {
            return;
        }
        self.cursor -= 1;
        let byte_pos = self.byte_pos();
        let end = self.buffer[byte_pos..]
        .char_indices()
        .nth(1)
        .map(|(i, _)| byte_pos + i)
        .unwrap_or(self.buffer.len());
        self.buffer.replace_range(byte_pos..end, "");
    }

    pub fn delete(&mut self) {
        let byte_pos = self.byte_pos();
        if byte_pos >= self.buffer.len() {
            return;
        }
        let end = self.buffer[byte_pos..]
        .char_indices()
        .nth(1)
        .map(|(i, _)| byte_pos + i)
        .unwrap_or(self.buffer.len());
        self.buffer.replace_range(byte_pos..end, "");
    }

    pub fn move_left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn move_right(&mut self) {
        let max = self.buffer.chars().count();
        if self.cursor < max {
            self.cursor += 1;
        }
    }

    pub fn move_home(&mut self) {
        self.cursor = 0;
    }

    pub fn move_end(&mut self) {
        self.cursor = self.buffer.chars().count();
    }

    fn byte_pos(&self) -> usize {
        self.buffer
        .char_indices()
        .nth(self.cursor)
        .map(|(i, _)| i)
        .unwrap_or(self.buffer.len())
    }
}

/// Parse a create command. Returns (is_dir, name).
///
/// - `d=foo`  -> (true, "foo")    directory
/// - `f=foo`  -> (false, "foo")   file (explicit)
/// - `foo/`   -> (true, "foo")    yazi-style trailing slash
/// - `foo`    -> (false, "foo")   default to file
pub fn parse_create(input: &str) -> Option<(bool, String)> {
    let s = input.trim();
    if s.is_empty() {
        return None;
    }
    if let Some(rest) = s.strip_prefix("d=") {
        let name = rest.trim().trim_end_matches('/');
        if name.is_empty() {
            return None;
        }
        return Some((true, name.to_string()));
    }
    if let Some(rest) = s.strip_prefix("f=") {
        let name = rest.trim();
        if name.is_empty() {
            return None;
        }
        return Some((false, name.to_string()));
    }
    if let Some(name) = s.strip_suffix('/') {
        if name.is_empty() {
            return None;
        }
        return Some((true, name.to_string()));
    }
    Some((false, s.to_string()))
}
