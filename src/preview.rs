use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::sync::OnceLock;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style as SynStyle, Theme, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

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

    // Try text preview (with syntax highlighting if possible)
    match text_preview(path) {
        Ok(lines) => Preview::Text(lines),
        Err(_) => Preview::Binary {
            size: metadata.len(),
        },
    }
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
