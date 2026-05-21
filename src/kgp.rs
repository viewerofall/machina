use anyhow::Result;
use base64::Engine;
use image::{imageops::FilterType, GenericImageView};
use std::io::Write;
use std::path::Path;

/// Returns true if we're running in a Kitty terminal (supports graphics protocol).
pub fn is_kitty() -> bool {
    std::env::var("KITTY_WINDOW_ID").is_ok()
        || std::env::var("TERM")
            .map(|t| t == "xterm-kitty")
            .unwrap_or(false)
}

/// Approximate pixel size of a single terminal cell (Kitty default).
const CELL_PX_W: u32 = 8;
const CELL_PX_H: u32 = 16;

/// Resize image to fit (cell_w x cell_h) terminal cells, preserving aspect.
pub fn resize_for_cells(path: &Path, cell_w: u16, cell_h: u16) -> Result<Vec<u8>> {
    let max_w = (cell_w as u32).saturating_mul(CELL_PX_W);
    let max_h = (cell_h as u32).saturating_mul(CELL_PX_H);

    let img = image::open(path)?;
    let (w, h) = img.dimensions();

    let resized = if w <= max_w && h <= max_h {
        img
    } else {
        img.resize(max_w.max(1), max_h.max(1), FilterType::Triangle)
    };

    // Encode as PNG
    let mut buf = Vec::with_capacity(64 * 1024);
    let dyn_img = resized.into_rgba8();
    let (rw, rh) = dyn_img.dimensions();
    image::codecs::png::PngEncoder::new(&mut buf).encode_with(
        |encoder, w, h| {
            use image::ImageEncoder;
            encoder.write_image(&dyn_img, w, h, image::ExtendedColorType::Rgba8)
        },
        rw,
        rh,
    )?;
    Ok(buf)
}

trait PngEncodeExt {
    fn encode_with<F>(self, f: F, w: u32, h: u32) -> Result<()>
    where
        F: FnOnce(Self, u32, u32) -> image::ImageResult<()>,
        Self: Sized;
}

impl<W: Write> PngEncodeExt for image::codecs::png::PngEncoder<W> {
    fn encode_with<F>(self, f: F, w: u32, h: u32) -> Result<()>
    where
        F: FnOnce(Self, u32, u32) -> image::ImageResult<()>,
        Self: Sized,
    {
        f(self, w, h).map_err(|e| anyhow::anyhow!(e))
    }
}

/// Display a PNG byte buffer at cursor position using Kitty graphics protocol.
/// Caller is responsible for positioning the cursor first.
pub fn display_kitty<W: Write>(out: &mut W, png: &[u8]) -> Result<()> {
    let b64 = base64::engine::general_purpose::STANDARD.encode(png);
    let bytes = b64.as_bytes();

    let chunk_size = 4096;
    let chunks: Vec<&[u8]> = bytes.chunks(chunk_size).collect();
    let count = chunks.len();

    for (i, chunk) in chunks.iter().enumerate() {
        let is_last = i == count - 1;
        let m_flag = if is_last { 0 } else { 1 };

        if i == 0 {
            // First chunk: full command
            write!(
                out,
                "\x1b_Ga=T,f=100,m={};{}\x1b\\",
                m_flag,
                std::str::from_utf8(chunk)?
            )?;
        } else {
            // Continuation
            write!(
                out,
                "\x1b_Gm={};{}\x1b\\",
                m_flag,
                std::str::from_utf8(chunk)?
            )?;
        }
    }
    out.flush()?;
    Ok(())
}

/// Delete all displayed Kitty images (cleanup).
pub fn clear_kitty<W: Write>(out: &mut W) -> Result<()> {
    write!(out, "\x1b_Ga=d\x1b\\")?;
    out.flush()?;
    Ok(())
}
