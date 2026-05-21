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
}

#[derive(Deserialize, Default)]
struct RawConfig {
    #[serde(default)]
    general: RawGeneral,
    #[serde(default)]
    bookmarks: HashMap<String, String>,
    #[serde(default)]
    openers: HashMap<String, String>,
}

#[derive(Deserialize, Default)]
struct RawGeneral {
    #[serde(default)]
    show_hidden: bool,
    #[serde(default = "default_true")]
    confirm_delete: bool,
    #[serde(default = "default_editor")]
    editor: String,
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

        for (k, v) in self.bookmarks {
            cfg.bookmarks.insert(k, expand_tilde(&v));
        }
        for (k, v) in self.openers {
            cfg.openers.insert(k.to_lowercase(), v);
        }
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
