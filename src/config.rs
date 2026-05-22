use anyhow::Result;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub show_hidden: bool,
    pub confirm_delete: bool,
    pub editor: String,
    pub bookmarks: HashMap<String, PathBuf>,
    pub openers: HashMap<String, String>,
    pub theme: crate::theme::Theme,
    pub respect_gitignore: bool,
    pub icons: crate::icons::IconMode,
}

#[derive(Deserialize, Default)]
struct RawConfig {
    #[serde(default)]
    general: RawGeneral,
    #[serde(default)]
    bookmarks: HashMap<String, String>,
    #[serde(default)]
    openers: HashMap<String, String>,
    #[serde(default)]
    theme: RawTheme,
}

#[derive(Deserialize, Default)]
struct RawGeneral {
    #[serde(default)]
    show_hidden: bool,
    #[serde(default = "default_true")]
    confirm_delete: bool,
    #[serde(default = "default_editor")]
    editor: String,
    #[serde(default = "default_true")]
    respect_gitignore: bool,
    #[serde(default = "default_icons")]
    icons: String,
}

fn default_icons() -> String {
    // "image" mode requires kitty 1.24+. Older versions don't support
    // Unicode placeholder graphics. Default to nerd for compatibility.
    "nerd".to_string()
}

#[derive(Deserialize, Default)]
struct RawTheme {
    bg: Option<String>,
    fg: Option<String>,
    accent: Option<String>,
    dim: Option<String>,
    visual_bg: Option<String>,
    dir_fg: Option<String>,
    file_fg: Option<String>,
    error_fg: Option<String>,
    warn_fg: Option<String>,
    git_ignored_fg: Option<String>,
    symlink_fg: Option<String>,
}

fn default_true() -> bool {
    true
}

fn default_editor() -> String {
    std::env::var("EDITOR").unwrap_or_else(|_| "nvim".to_string())
}

impl Default for Config {
    fn default() -> Self {
        let mut bookmarks = HashMap::new();
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));

        bookmarks.insert("h".into(), home.clone());
        bookmarks.insert("d".into(), home.join("Downloads"));
        bookmarks.insert("D".into(), home.join("Documents"));
        bookmarks.insert("c".into(), home.join(".config"));
        bookmarks.insert("r".into(), PathBuf::from("/"));

        let mut openers = HashMap::new();
        let editor = default_editor();
        for ext in [
            "rs", "lua", "md", "toml", "py", "sh", "c", "h", "zig", "go",
            "js", "ts", "json", "yaml", "yml", "txt", "conf", "ini", "log",
            "html", "css", "xml",
        ] {
            openers.insert(ext.to_string(), editor.clone());
        }
        for ext in ["png", "jpg", "jpeg", "gif", "webp", "svg", "bmp", "ico"] {
            openers.insert(ext.to_string(), "imv".to_string());
        }
        for ext in ["mp4", "mkv", "webm", "avi", "mov", "flv"] {
            openers.insert(ext.to_string(), "mpv".to_string());
        }
        for ext in ["mp3", "flac", "wav", "ogg", "opus", "m4a"] {
            openers.insert(ext.to_string(), "mpv".to_string());
        }
        openers.insert("pdf".into(), "zathura".into());

        Self {
            show_hidden: false,
            confirm_delete: true,
            editor,
            bookmarks,
            openers,
            theme: crate::theme::Theme::default(),
            respect_gitignore: true,
            icons: crate::icons::IconMode::Nerd,
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let path = Self::config_path();
        if !path.exists() {
            return Config::default();
        }
        match std::fs::read_to_string(&path) {
            Ok(contents) => match toml::from_str::<RawConfig>(&contents) {
                Ok(raw) => raw.into_config(),
                Err(_) => Config::default(),
            },
            Err(_) => Config::default(),
        }
    }

    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("machina")
            .join("config.toml")
    }

    /// Persist a single bookmark key into the user config file. Lossy: rewrites just the
    /// [bookmarks] table by appending/replacing the key. Keeps everything else intact.
    pub fn save_bookmark(key: &str, path: &std::path::Path) -> Result<()> {
        let cfg_path = Self::config_path();
        if let Some(parent) = cfg_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut existing = std::fs::read_to_string(&cfg_path).unwrap_or_else(|_| String::new());
        let line = format!("{} = \"{}\"", key, path.display());

        // If [bookmarks] section exists, look for key=
        let needle_section = "[bookmarks]";
        if let Some(sec_pos) = existing.find(needle_section) {
            // Find boundaries of section
            let after = &existing[sec_pos + needle_section.len()..];
            let next_section = after.find("\n[").map(|p| sec_pos + needle_section.len() + p);
            let section_end = next_section.unwrap_or(existing.len());
            let section_body = &existing[sec_pos + needle_section.len()..section_end];

            let key_prefix = format!("\n{} =", key);
            let key_prefix_alt = format!("\n{} = ", key);
            if section_body.contains(&key_prefix) || section_body.contains(&key_prefix_alt) {
                // Replace the line
                let mut out = String::new();
                for l in existing.lines() {
                    let trimmed = l.trim_start();
                    if trimmed.starts_with(&format!("{} =", key)) || trimmed.starts_with(&format!("{}=", key)) {
                        out.push_str(&line);
                        out.push('\n');
                    } else {
                        out.push_str(l);
                        out.push('\n');
                    }
                }
                existing = out;
            } else {
                // Insert after [bookmarks]
                let insertion = sec_pos + needle_section.len();
                existing.insert_str(insertion, &format!("\n{}", line));
            }
        } else {
            if !existing.ends_with('\n') && !existing.is_empty() {
                existing.push('\n');
            }
            existing.push_str(&format!("\n[bookmarks]\n{}\n", line));
        }

        std::fs::write(&cfg_path, existing)?;
        Ok(())
    }

    pub fn ensure_default() -> Result<()> {
        let path = Self::config_path();
        if path.exists() {
            return Ok(());
        }
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, DEFAULT_CONFIG_TOML)?;
        Ok(())
    }
}

impl RawConfig {
    fn into_config(self) -> Config {
        let mut cfg = Config::default();
        cfg.show_hidden = self.general.show_hidden;
        cfg.confirm_delete = self.general.confirm_delete;
        cfg.editor = self.general.editor;
        cfg.respect_gitignore = self.general.respect_gitignore;
        cfg.icons = crate::icons::parse_mode(&self.general.icons);

        for (k, v) in self.bookmarks {
            cfg.bookmarks.insert(k, expand_tilde(&v));
        }
        for (k, v) in self.openers {
            cfg.openers.insert(k.to_lowercase(), v);
        }

        // Theme overrides
        let t = &mut cfg.theme;
        if let Some(c) = self.theme.bg.and_then(|s| crate::theme::parse_hex(&s)) { t.bg = c; }
        if let Some(c) = self.theme.fg.and_then(|s| crate::theme::parse_hex(&s)) { t.fg = c; t.file_fg = c; }
        if let Some(c) = self.theme.accent.and_then(|s| crate::theme::parse_hex(&s)) { t.accent = c; t.dir_fg = c; t.ok_fg = c; }
        if let Some(c) = self.theme.dim.and_then(|s| crate::theme::parse_hex(&s)) { t.dim = c; }
        if let Some(c) = self.theme.visual_bg.and_then(|s| crate::theme::parse_hex(&s)) { t.visual_bg = c; }
        if let Some(c) = self.theme.dir_fg.and_then(|s| crate::theme::parse_hex(&s)) { t.dir_fg = c; }
        if let Some(c) = self.theme.file_fg.and_then(|s| crate::theme::parse_hex(&s)) { t.file_fg = c; }
        if let Some(c) = self.theme.error_fg.and_then(|s| crate::theme::parse_hex(&s)) { t.error_fg = c; }
        if let Some(c) = self.theme.warn_fg.and_then(|s| crate::theme::parse_hex(&s)) { t.warn_fg = c; }
        if let Some(c) = self.theme.git_ignored_fg.and_then(|s| crate::theme::parse_hex(&s)) { t.git_ignored_fg = c; }
        if let Some(c) = self.theme.symlink_fg.and_then(|s| crate::theme::parse_hex(&s)) { t.symlink_fg = c; }

        cfg
    }
}

fn expand_tilde(s: &str) -> PathBuf {
    if let Some(rest) = s.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest);
        }
    }
    PathBuf::from(s)
}

const DEFAULT_CONFIG_TOML: &str = r##"# machina config (~/.config/machina/config.toml)

[general]
show_hidden = false
confirm_delete = true
editor = "nvim"
respect_gitignore = true     # treat .gitignore matches as hidden (toggle with `.`)
# Icon mode: "nerd" (Nerd-Font glyph), "image" (kitty 1.24+ graphics),
#           "ascii" (plain text marker), "off" (no icon column).
# Note: "image" requires kitty v1.24+. Falls back to "nerd" on older kitty.
icons = "nerd"

# Theme — hex colors. All optional; omitted keys fall back to OneShot defaults.
[theme]
# bg            = "#0a0010"
# fg            = "#c792ea"
# accent        = "#00e5c8"
# dim           = "#6c7086"
# visual_bg     = "#1c1032"
# dir_fg        = "#00e5c8"
# file_fg       = "#c792ea"
# error_fg      = "#ff5555"
# warn_fg       = "#ffb86c"
# git_ignored_fg= "#464a60"
# symlink_fg    = "#ffd170"

# Goto bookmarks: press `g <key>` to jump
[bookmarks]
h = "~"
d = "~/Downloads"
D = "~/Documents"
c = "~/.config"
r = "/"
p = "~/Projects"

# Extension -> command. Used by Enter/l on a file.
[openers]
rs   = "nvim"
lua  = "nvim"
md   = "nvim"
toml = "nvim"
py   = "nvim"
sh   = "nvim"
txt  = "nvim"

png  = "imv"
jpg  = "imv"
jpeg = "imv"
gif  = "imv"
webp = "imv"

mp4 = "mpv"
mkv = "mpv"
mp3 = "mpv"

pdf = "zathura"
"##;
