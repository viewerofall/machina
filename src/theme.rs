use ratatui::style::Color;

// OneShot TWM aesthetic
pub const BG: Color = Color::Rgb(0x0a, 0x00, 0x10);        // #0a0010 deep purple
pub const FG: Color = Color::Rgb(0xc7, 0x92, 0xea);        // #c792ea light purple
pub const ACCENT: Color = Color::Rgb(0x00, 0xe5, 0xc8);    // #00e5c8 cyan

pub const DIM: Color = Color::Rgb(0x6c, 0x70, 0x86);       // dim gray for borders/inactive
pub const HIGHLIGHT_BG: Color = Color::Rgb(0x32, 0x14, 0x52); // selection background
pub const VISUAL_BG: Color = Color::Rgb(0x1c, 0x10, 0x32); // visual mode selection

pub const DIR_FG: Color = ACCENT;                          // directories in cyan
pub const FILE_FG: Color = FG;                             // files in purple

pub const ERROR_FG: Color = Color::Rgb(0xff, 0x55, 0x55);
pub const WARN_FG: Color = Color::Rgb(0xff, 0xb8, 0x6c);
pub const OK_FG: Color = ACCENT;
