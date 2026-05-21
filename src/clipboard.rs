use anyhow::Result;

/// Copy a text string to the system clipboard.
/// Uses arboard which works on Wayland (wl-clipboard) and X11.
pub fn copy(text: &str) -> Result<()> {
    let mut clipboard = arboard::Clipboard::new()?;
    clipboard.set_text(text.to_string())?;
    Ok(())
}
