//! Icon resolution. Three modes:
//!   * `Nerd`  — Nerd Font PUA glyphs (requires Nerd-Font kitty config)
//!   * `Image` — kitty graphics protocol unicode-placeholder sprites
//!   * `Ascii` — plain text marker (`>` for dirs, ` ` for files)
//!   * `Off`   — no icon column at all
//!
//! Image mode dispatches to `icon_sprites::id_for(kind)` where `kind` is a
//! short keyword like "rs" / "folder_dl".

use std::path::Path;
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconMode {
    Off,
    Ascii,
    Nerd,
    Image,
}

static MODE: OnceLock<IconMode> = OnceLock::new();

pub fn set_mode(m: IconMode) {
    let _ = MODE.set(m);
}

pub fn mode() -> IconMode {
    *MODE.get().unwrap_or(&IconMode::Nerd)
}

pub fn parse_mode(s: &str) -> IconMode {
    match s.to_ascii_lowercase().as_str() {
        "off" | "none" | "false" => IconMode::Off,
        "ascii" | "text"         => IconMode::Ascii,
        "image" | "kitty"        => IconMode::Image,
        _                        => IconMode::Nerd,
    }
}

/// Resolved icon for a single row.
pub enum Icon {
    /// Nothing to draw (off mode).
    None,
    /// Plain text glyph — render in current style.
    Glyph(&'static str),
    /// KGP sprite — emit U+10EEEE with this image ID encoded in fg color.
    Sprite(u32),
}

pub fn resolve(name: &str, is_dir: bool, is_symlink: bool) -> Icon {
    match mode() {
        IconMode::Off => Icon::None,
        IconMode::Ascii => Icon::Glyph(ascii_for(is_dir, is_symlink)),
        IconMode::Nerd => Icon::Glyph(nerd_for(name, is_dir, is_symlink)),
        IconMode::Image => {
            let kind = sprite_kind_for(name, is_dir, is_symlink);
            let id = crate::icon_sprites::id_for(kind);
            if id == 0 {
                Icon::Glyph(nerd_for(name, is_dir, is_symlink))
            } else {
                Icon::Sprite(id)
            }
        }
    }
}

fn ascii_for(is_dir: bool, is_symlink: bool) -> &'static str {
    if is_symlink { "@" } else if is_dir { ">" } else { " " }
}

// ---------------------------------------------------------------------------
// Sprite-kind keywords — keep in sync with assets/icons/*.png filenames.
// ---------------------------------------------------------------------------
fn sprite_kind_for(name: &str, is_dir: bool, is_symlink: bool) -> &'static str {
    if is_symlink {
        return "symlink";
    }
    if is_dir {
        return sprite_dir_kind(name);
    }
    let lower = name.to_ascii_lowercase();
    if let Some(k) = sprite_special_file(&lower) {
        return k;
    }
    let ext = Path::new(&lower).extension().and_then(|e| e.to_str()).unwrap_or("");
    sprite_ext_kind(ext)
}

fn sprite_dir_kind(name: &str) -> &'static str {
    match name.to_ascii_lowercase().as_str() {
        "downloads"                              => "folder_dl",
        "documents" | "doc" | "docs"             => "folder_docs",
        "pictures" | "images" | "img" | "photos" => "folder_pics",
        "videos" | "video" | "movies"            => "folder_vid",
        "music" | "audio" | "songs"              => "folder_music",
        "projects" | "code" | "dev" | "src"      => "folder_proj",
        ".config" | "config"                     => "folder_cfg",
        ".git" | ".github"                       => "folder_git",
        "node_modules"                           => "folder_node",
        "target" | "build" | "dist" | "out"      => "folder_proj",
        ".cache" | "cache"                       => "folder_cache",
        ".trash" | "trash"                       => "folder_trash",
        "home"                                   => "folder_home",
        _                                        => "folder",
    }
}

fn sprite_special_file(name: &str) -> Option<&'static str> {
    Some(match name {
        "readme" | "readme.md" | "readme.txt"               => "readme",
        "license" | "license.md" | "license.txt"            => "license",
        "makefile" | "gnumakefile"                          => "makefile",
        "dockerfile" | ".dockerignore"                      => "dockerfile",
        "cargo.toml" | "cargo.lock"                         => "cargo",
        ".gitignore" | ".gitattributes" | ".gitmodules"     => "gitignore",
        "package.json" | "package-lock.json" | "yarn.lock"  => "package",
        ".env" | ".env.local" | ".env.example"              => "env",
        _ => return None,
    })
}

fn sprite_ext_kind(ext: &str) -> &'static str {
    match ext {
        "rs" => "rs",
        "c" | "h" => "c",
        "cpp" | "cc" | "cxx" | "hpp" | "hxx" => "cpp",
        "py" | "pyc" | "pyo" => "py",
        "js" | "mjs" | "cjs" | "jsx" => "js",
        "ts" | "tsx" => "ts",
        "html" | "htm" => "html",
        "css" | "scss" | "sass" => "css",
        "lua" => "lua",
        "go" => "go",
        "zig" => "zig",
        "java" | "jar" | "class" | "kt" | "kts" => "java",
        "rb" => "rb",
        "php" => "php",
        "sh" | "bash" | "zsh" | "fish" => "sh",
        "md" | "markdown" => "md",
        "toml" => "toml",
        "json" | "json5" => "json",
        "yaml" | "yml" => "yaml",
        "xml" => "xml",
        "ini" | "conf" | "cfg" => "conf",
        "log" => "log",
        "txt" | "rst" | "tex" => "txt",
        "zip" | "tar" | "gz" | "bz2" | "xz" | "zst" | "7z" | "rar" | "tgz" | "tbz2" => "archive",
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "ico" | "tiff" | "svg" => "image",
        "mp4" | "mkv" | "webm" | "avi" | "mov" | "flv" | "wmv" | "m4v" => "video",
        "mp3" | "flac" | "wav" | "ogg" | "opus" | "m4a" | "aac" => "audio",
        "pdf" => "pdf",
        "exe" | "bin" | "appimage" => "exe",
        "iso" | "img" | "deb" | "rpm" | "pkg" => "iso",
        "ttf" | "otf" | "woff" | "woff2" => "font",
        "lock" => "lock",
        _ => "file_generic",
    }
}

// ---------------------------------------------------------------------------
// Nerd-font glyphs (mode = "nerd")
// ---------------------------------------------------------------------------
pub fn nerd_for(name: &str, is_dir: bool, is_symlink: bool) -> &'static str {
    if is_symlink {
        return "";
    }
    if is_dir {
        return nerd_dir(name);
    }
    let lower = name.to_ascii_lowercase();
    if let Some(i) = nerd_special_file(&lower) {
        return i;
    }
    let ext = Path::new(&lower).extension().and_then(|e| e.to_str()).unwrap_or("");
    nerd_ext(ext)
}

fn nerd_dir(name: &str) -> &'static str {
    match name.to_ascii_lowercase().as_str() {
        "downloads"                              => "",
        "documents" | "doc" | "docs"             => "",
        "pictures" | "images" | "img" | "photos" => "",
        "videos" | "video" | "movies"            => "",
        "music" | "audio" | "songs"              => "",
        "desktop"                                => "",
        "projects" | "code" | "dev" | "src"      => "",
        ".config" | "config"                     => "",
        ".git"                                   => "",
        ".github"                                => "",
        "node_modules"                           => "",
        "target" | "build" | "dist" | "out"      => "",
        ".cache" | "cache"                       => "",
        ".trash" | "trash"                       => "",
        "home"                                   => "",
        "public"                                 => "",
        "templates"                              => "",
        _                                        => "",
    }
}

fn nerd_special_file(name: &str) -> Option<&'static str> {
    Some(match name {
        "readme" | "readme.md" | "readme.txt"               => "",
        "license" | "license.md" | "license.txt"            => "",
        "makefile" | "gnumakefile"                          => "",
        "dockerfile" | ".dockerignore"                      => "",
        "cargo.toml" | "cargo.lock"                         => "",
        ".gitignore" | ".gitattributes" | ".gitmodules"     => "",
        "package.json" | "package-lock.json"                => "",
        "yarn.lock"                                         => "",
        ".env" | ".env.local" | ".env.example"              => "",
        ".bashrc" | ".zshrc" | ".profile" | ".bash_profile" => "",
        "kitty.conf"                                        => "",
        _ => return None,
    })
}

fn nerd_ext(ext: &str) -> &'static str {
    match ext {
        "rs"                                    => "",
        "c" | "h"                               => "",
        "cpp" | "cc" | "cxx" | "hpp" | "hxx"    => "",
        "py" | "pyc" | "pyo"                    => "",
        "js" | "mjs" | "cjs"                    => "",
        "ts" | "tsx"                            => "",
        "jsx"                                   => "",
        "html" | "htm"                          => "",
        "css"                                   => "",
        "scss" | "sass"                         => "",
        "lua"                                   => "",
        "go"                                    => "",
        "zig"                                   => "",
        "java" | "jar" | "class"                => "",
        "kt" | "kts"                            => "",
        "rb"                                    => "",
        "php"                                   => "",
        "swift"                                 => "",
        "sh" | "bash" | "zsh" | "fish"          => "",
        "vim" | "nvim"                          => "",
        "el"                                    => "",
        "hs"                                    => "",
        "ml" | "mli"                            => "",
        "ex" | "exs"                            => "",
        "dart"                                  => "",
        "json" | "json5"                        => "",
        "toml"                                  => "",
        "yaml" | "yml"                          => "",
        "xml"                                   => "󰗀",
        "md" | "markdown"                       => "",
        "tex" | "latex"                         => "",
        "csv" | "tsv"                           => "",
        "ini" | "conf" | "cfg"                  => "",
        "log"                                   => "",
        "txt"                                   => "",
        "zip" | "tar" | "gz" | "bz2" | "xz"
        | "zst" | "7z" | "rar" | "tgz" | "tbz2" => "",
        "png" | "jpg" | "jpeg" | "gif" | "webp"
        | "bmp" | "ico" | "tiff" | "svg"        => "",
        "mp4" | "mkv" | "webm" | "avi" | "mov"
        | "flv" | "wmv" | "m4v"                 => "",
        "mp3" | "flac" | "wav" | "ogg" | "opus"
        | "m4a" | "aac"                         => "",
        "pdf"                                   => "",
        "doc" | "docx"                          => "",
        "xls" | "xlsx"                          => "",
        "ppt" | "pptx"                          => "",
        "exe" | "bin" | "appimage"              => "",
        "iso" | "img"                           => "",
        "deb"                                   => "",
        "rpm"                                   => "",
        "pkg"                                   => "",
        "ttf" | "otf" | "woff" | "woff2"        => "",
        "lock"                                  => "",
        ""                                      => "",
        _                                       => "",
    }
}

/// Legacy entry point kept for any callers still using the old signature.
/// Returns a Nerd-font glyph regardless of current mode.
pub fn icon_for(name: &str, is_dir: bool, is_symlink: bool) -> &'static str {
    nerd_for(name, is_dir, is_symlink)
}
