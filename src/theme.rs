use ratatui::style::Color;
use std::sync::OnceLock;

#[derive(Clone, Debug)]
pub struct Theme {
    pub bg: Color,
    pub fg: Color,
    pub accent: Color,
    pub dim: Color,
    pub highlight_bg: Color,
    pub visual_bg: Color,
    pub dir_fg: Color,
    pub file_fg: Color,
    pub error_fg: Color,
    pub warn_fg: Color,
    pub ok_fg: Color,
    pub git_ignored_fg: Color,
    pub symlink_fg: Color,
}

impl Default for Theme {
    fn default() -> Self {
        // OneShot TWM
        let bg = Color::Rgb(0x0a, 0x00, 0x10);
        let fg = Color::Rgb(0xc7, 0x92, 0xea);
        let accent = Color::Rgb(0x00, 0xe5, 0xc8);
        Theme {
            bg,
            fg,
            accent,
            dim: Color::Rgb(0x6c, 0x70, 0x86),
            highlight_bg: Color::Rgb(0x32, 0x14, 0x52),
            visual_bg: Color::Rgb(0x1c, 0x10, 0x32),
            dir_fg: accent,
            file_fg: fg,
            error_fg: Color::Rgb(0xff, 0x55, 0x55),
            warn_fg: Color::Rgb(0xff, 0xb8, 0x6c),
            ok_fg: accent,
            git_ignored_fg: Color::Rgb(0x46, 0x4a, 0x60),
            symlink_fg: Color::Rgb(0xff, 0xd1, 0x70),
        }
    }
}

pub fn parse_hex(s: &str) -> Option<Color> {
    let s = s.trim().trim_start_matches('#');
    if s.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&s[0..2], 16).ok()?;
    let g = u8::from_str_radix(&s[2..4], 16).ok()?;
    let b = u8::from_str_radix(&s[4..6], 16).ok()?;
    Some(Color::Rgb(r, g, b))
}

static THEME: OnceLock<Theme> = OnceLock::new();

pub fn set(t: Theme) {
    let _ = THEME.set(t);
}

fn t() -> &'static Theme {
    THEME.get_or_init(Theme::default)
}

pub fn bg() -> Color { t().bg }
pub fn fg() -> Color { t().fg }
pub fn accent() -> Color { t().accent }
pub fn dim() -> Color { t().dim }
#[allow(dead_code)]
pub fn highlight_bg() -> Color { t().highlight_bg }
pub fn visual_bg() -> Color { t().visual_bg }
pub fn dir_fg() -> Color { t().dir_fg }
pub fn file_fg() -> Color { t().file_fg }
pub fn error_fg() -> Color { t().error_fg }
pub fn warn_fg() -> Color { t().warn_fg }
#[allow(dead_code)]
pub fn ok_fg() -> Color { t().ok_fg }
pub fn git_ignored_fg() -> Color { t().git_ignored_fg }
pub fn symlink_fg() -> Color { t().symlink_fg }
