use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::sync::OnceLock;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style as SynStyle, Theme, ThemeSet};
use syntect::parsing::SyntaxSet;

const MAX_LINES: usize = 200;
const MAX_BYTES_PER_LINE: usize = 1024;

static SYNTAX_SET: OnceLock<SyntaxSet> = OnceLock::new();
static THEME: OnceLock<Theme> = OnceLock::new();

fn syntax_set() -> &'static SyntaxSet {
    SYNTAX_SET.get_or_init(SyntaxSet::load_defaults_newlines)
}

fn theme() -> &'static Theme {
    THEME.get_or_init(|| {
        let ts = ThemeSet::load_defaults();
        // Match OneShot aesthetic - dark purple bg
        ts.themes
        .get("base16-mocha.dark")
        .or_else(|| ts.themes.get("base16-ocean.dark"))
        .cloned()
        .unwrap_or_else(|| ts.themes.values().next().unwrap().clone())
    })
}

pub enum Preview {
    Text(Vec<Line<'static>>),
    Directory(Vec<String>, usize),
    Image { width: u32, height: u32, size: u64 },
    Binary { size: u64 },
    Archive { entries: Vec<String>, total: usize, size: u64, kind: &'static str },
    Empty,
    Error(String),
}

pub fn get_preview(path: &Path) -> Preview {
    if path.is_dir() {
        return dir_preview(path);
    }

    let metadata = match std::fs::metadata(path) {
        Ok(m) => m,
        Err(e) => return Preview::Error(format!("Cannot stat: {}", e)),
    };

    if metadata.len() == 0 {
        return Preview::Empty;
    }

    let ext = path
    .extension()
    .and_then(|e| e.to_str())
    .map(str::to_lowercase);

    // Image preview (just metadata for TUI)
    if let Some(ref e) = ext {
        if matches!(
            e.as_str(),
                    "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "svg" | "ico"
        ) {
            if let Ok(img) = image::open(path) {
                return Preview::Image {
                    width: img.width(),
                    height: img.height(),
                    size: metadata.len(),
                };
            }
        }
    }

    // Archive preview
    if let Some(ref e) = ext {
        match e.as_str() {
            "zip" => {
                if let Some(p) = zip_preview(path, metadata.len()) {
                    return p;
                }
            }
            "tar" | "gz" | "tgz" | "bz2" | "tbz2" | "xz" => {
                if let Some(p) = tar_preview(path, metadata.len(), e) {
                    return p;
                }
            }
            _ => {}
        }
    }

    // Try text preview (with syntax highlighting if possible)
    match text_preview(path) {
        Ok(lines) => Preview::Text(lines),
        Err(_) => Preview::Binary {
            size: metadata.len(),
        },
    }
}

fn zip_preview(path: &Path, size: u64) -> Option<Preview> {
    let file = std::fs::File::open(path).ok()?;
    let mut zip = zip::ZipArchive::new(file).ok()?;
    let total = zip.len();
    let mut entries = Vec::with_capacity(total.min(MAX_LINES));
    for i in 0..total.min(MAX_LINES) {
        if let Ok(z) = zip.by_index(i) {
            let prefix = if z.is_dir() { "" } else { "" };
            entries.push(format!("{}  {}", prefix, z.name()));
        }
    }
    Some(Preview::Archive {
        entries,
        total,
        size,
        kind: "zip",
    })
}

fn tar_preview(path: &Path, size: u64, ext: &str) -> Option<Preview> {
    let file = std::fs::File::open(path).ok()?;
    let (entries, total, kind) = match ext {
        "gz" | "tgz" => {
            let mut ar = tar::Archive::new(flate2::read::GzDecoder::new(file));
            let (e, t) = collect_tar_paths(&mut ar);
            (e, t, "tar.gz")
        }
        _ => {
            let mut ar = tar::Archive::new(file);
            let (e, t) = collect_tar_paths(&mut ar);
            (e, t, "tar")
        }
    };
    Some(Preview::Archive { entries, total, size, kind })
}

fn collect_tar_paths<R: std::io::Read>(ar: &mut tar::Archive<R>) -> (Vec<String>, usize) {
    let mut out = Vec::with_capacity(MAX_LINES);
    let mut total = 0;
    let Ok(iter) = ar.entries() else {
        return (out, total);
    };
    for e in iter {
        total += 1;
        if out.len() < MAX_LINES {
            if let Ok(ent) = e {
                let is_dir = ent.header().entry_type().is_dir();
                if let Ok(p) = ent.path() {
                    let prefix = if is_dir { "" } else { "" };
                    out.push(format!("{}  {}", prefix, p.display()));
                }
            }
        }
    }
    (out, total)
}

fn dir_preview(path: &Path) -> Preview {
    match std::fs::read_dir(path) {
        Ok(entries) => {
            let mut items: Vec<_> = entries
            .filter_map(|e| e.ok())
            .map(|e| {
                let p = e.path();
                let name = p
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("?")
                .to_string();
                let prefix = if p.is_dir() { "📁 " } else { "  " };
                format!("{}{}", prefix, name)
            })
            .collect();
            items.sort();
            let total = items.len();
            items.truncate(MAX_LINES);
            Preview::Directory(items, total)
        }
        Err(e) => Preview::Error(format!("Cannot read: {}", e)),
    }
}

fn text_preview(path: &Path) -> Result<Vec<Line<'static>>, std::io::Error> {
    let file = std::fs::File::open(path)?;
    let reader = BufReader::new(file);

    let ss = syntax_set();
    let th = theme();

    let syntax = ss
    .find_syntax_for_file(path)
    .ok()
    .flatten()
    .unwrap_or_else(|| ss.find_syntax_plain_text());

    let mut highlighter = HighlightLines::new(syntax, th);
    let mut lines: Vec<Line<'static>> = Vec::with_capacity(MAX_LINES);

    for (i, line_result) in reader.lines().enumerate() {
        if i >= MAX_LINES {
            break;
        }
        let mut line = line_result?;
        if line.len() > MAX_BYTES_PER_LINE {
            line.truncate(MAX_BYTES_PER_LINE);
            line.push_str("…");
        }

        // Binary check on first lines: if NULL byte exists, abort
        if line.as_bytes().contains(&0u8) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "binary",
            ));
        }

        let line_with_nl = format!("{}\n", line);
        let ranges = highlighter
        .highlight_line(&line_with_nl, ss)
        .unwrap_or_default();

        let spans: Vec<Span<'static>> = ranges
        .into_iter()
        .map(|(style, text)| {
            let color = syn_to_color(style);
            Span::styled(
                text.trim_end_matches('\n').to_string(),
                         Style::default().fg(color),
            )
        })
        .collect();

        lines.push(Line::from(spans));
    }

    Ok(lines)
}

fn syn_to_color(style: SynStyle) -> Color {
    Color::Rgb(style.foreground.r, style.foreground.g, style.foreground.b)
}

pub fn format_size(size: u64) -> String {
    if size < 1024 {
        format!("{} B", size)
    } else if size < 1024 * 1024 {
        format!("{:.1} KB", size as f64 / 1024.0)
    } else if size < 1024 * 1024 * 1024 {
        format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", size as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}
