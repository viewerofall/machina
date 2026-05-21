mod app;
mod archive;
mod clipboard;
mod config;
mod cwd_writer;
mod git;
mod input;
mod kgp;
mod opener;
mod ops;
mod preview;
mod shell;
mod theme;
mod tmux;
mod ui;
mod watcher;

use anyhow::Result;
use app::{App, HelpVisible, Mode, OpMode};
use config::Config;
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, KeyCode, KeyEvent, KeyEventKind,
        KeyModifiers,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use input::InputAction;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use watcher::FsWatcher;

#[tokio::main]
async fn main() -> Result<()> {
    let _ = Config::ensure_default();
    let config = Config::load();
    let start = std::env::args().nth(1).map(PathBuf::from);
    let mut app = App::new(start, config)?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    enable_raw_mode()?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let mut last_image_area: Option<ratatui::layout::Rect> = None;
    let kitty = kgp::is_kitty();

    let mut fs_watcher = FsWatcher::new().ok();
    refresh_watcher(&mut fs_watcher, &app);
    let mut last_watcher_refresh = Instant::now();

    loop {
        let mut image_area: Option<ratatui::layout::Rect> = None;
        terminal.draw(|f| {
            image_area = ui::draw(f, &app);
        })?;

        // Image overlay (kitty)
        let mut out = io::stdout();
        if last_image_area.is_some() && image_area != last_image_area {
            let _ = kgp::clear_kitty(&mut out);
        }
        if kitty {
            if let (Some(area), Some(entry)) = (image_area, app.current().hovered()) {
                if is_image_path(&entry.path) {
                    let path = entry.path.clone();
                    if let Ok(png) = kgp::resize_for_cells(&path, area.width, area.height) {
                        use crossterm::cursor::MoveTo;
                        use crossterm::QueueableCommand;
                        let _ = out.queue(MoveTo(area.x, area.y));
                        use std::io::Write;
                        let _ = out.flush();
                        let _ = kgp::display_kitty(&mut out, &png);
                    }
                }
            }
        }
        last_image_area = image_area;

        // Poll input
        if crossterm::event::poll(Duration::from_millis(100))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Release {
                    if !handle_key(&mut app, key, &mut terminal)? {
                        break;
                    }
                    // Foreground shell command queued? Run it now (we have &mut terminal).
                    if let Some(cmd) = app.pending_fg_shell.take() {
                        let cwd = app.current().path.clone();
                        suspend_and_run(&mut terminal, || shell::run(&cmd, &cwd))?;
                        let _ = app.current_mut().load();
                    }
                    refresh_watcher(&mut fs_watcher, &app);
                }
            }
        }

        // Periodically check watcher
        if last_watcher_refresh.elapsed() > Duration::from_millis(250) {
            if let Some(w) = fs_watcher.as_mut() {
                if w.drain() {
                    // Reload all visible folders
                    let active = app.active;
                    let split = app.split;
                    let _ = app.tabs[active].load();
                    if let Some(s) = split {
                        if let Some(t) = app.tabs.get_mut(s) {
                            let _ = t.load();
                        }
                    }
                }
            }
            last_watcher_refresh = Instant::now();
        }
    }

    // CD on exit
    let final_cwd = app.current().path.clone();

    // Cleanup terminal
    let mut out = io::stdout();
    let _ = kgp::clear_kitty(&mut out);
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
             LeaveAlternateScreen,
             DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    let _ = cwd_writer::write(&final_cwd);
    Ok(())
}

fn refresh_watcher(w: &mut Option<FsWatcher>, app: &App) {
    let Some(w) = w.as_mut() else { return };
    w.unwatch_all();
    let _ = w.watch(&app.current().path);
    if let Some(s) = app.split {
        if let Some(t) = app.tabs.get(s) {
            let _ = w.watch(&t.path);
        }
    }
}

fn is_image_path(path: &std::path::Path) -> bool {
    path.extension()
    .and_then(|e| e.to_str())
    .map(|e| e.to_lowercase())
    .map(|e| matches!(e.as_str(), "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp"))
    .unwrap_or(false)
}

type Term = Terminal<CrosstermBackend<io::Stdout>>;

fn handle_key(app: &mut App, key: KeyEvent, term: &mut Term) -> Result<bool> {
    // Modal dialogs first
    if app.help == HelpVisible::Shown {
        return handle_help(app, key);
    }
    if app.paste_dialog.is_some() {
        return handle_paste_dialog(app, key);
    }
    if app.confirm.is_some() {
        return handle_confirm(app, key);
    }
    if app.input.active {
        return handle_input(app, key);
    }
    match app.keybind.mode {
        Mode::Normal => handle_normal(app, key, term),
        Mode::Visual => handle_visual(app, key, term),
    }
}

fn handle_help(app: &mut App, key: KeyEvent) -> Result<bool> {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') => {
            app.help = HelpVisible::Hidden;
        }
        _ => {}
    }
    Ok(true)
}

fn handle_paste_dialog(app: &mut App, key: KeyEvent) -> Result<bool> {
    let Some(d) = app.paste_dialog.as_mut() else {
        return Ok(true);
    };
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.paste_dialog = None;
        }
        KeyCode::Left | KeyCode::Char('h') => {
            d.selected = d.selected.saturating_sub(1);
        }
        KeyCode::Right | KeyCode::Char('l') => {
            d.selected = (d.selected + 1).min(2);
        }
        KeyCode::Char('c') => d.selected = 0,
        KeyCode::Char('m') => d.selected = 1,
        KeyCode::Char('s') => d.selected = 2,
        KeyCode::Enter => {
            let mode = match d.selected {
                0 => OpMode::Copy,
                1 => OpMode::Cut,
                2 => OpMode::Link,
                _ => OpMode::Copy,
            };
            let dest = d.dest_override.clone();
            app.paste_dialog = None;
            ops::commit_paste(app, mode, dest)?;
        }
        _ => {}
    }
    Ok(true)
}

fn handle_confirm(app: &mut App, key: KeyEvent) -> Result<bool> {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') => ops::confirm_yes(app)?,
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => ops::confirm_no(app),
        _ => {}
    }
    Ok(true)
}

fn handle_input(app: &mut App, key: KeyEvent) -> Result<bool> {
    match key.code {
        KeyCode::Esc => {
            app.input.close();
            app.current_mut().clear_filter();
        }
        KeyCode::Enter => {
            let action = app.input.action;
            let value = app.input.buffer.clone();
            app.input.close();
            commit_input(app, action, value)?;
        }
        KeyCode::Backspace => {
            app.input.backspace();
            if app.input.action == InputAction::Search {
                let buf = app.input.buffer.clone();
                app.current_mut().set_filter(buf);
            }
        }
        KeyCode::Delete => {
            app.input.delete();
            if app.input.action == InputAction::Search {
                let buf = app.input.buffer.clone();
                app.current_mut().set_filter(buf);
            }
        }
        KeyCode::Left => app.input.move_left(),
        KeyCode::Right => app.input.move_right(),
        KeyCode::Home => app.input.move_home(),
        KeyCode::End => app.input.move_end(),
        KeyCode::Char(c) => {
            app.input.insert(c);
            // Only update filter during active search
            if app.input.action == InputAction::Search {
                let buf = app.input.buffer.clone();
                app.current_mut().set_filter(buf);
            }
        }
        _ => {}
    }
    Ok(true)
}

fn commit_input(app: &mut App, action: InputAction, value: String) -> Result<()> {
    match action {
        InputAction::Rename => {
            if value.is_empty() {
                return Ok(());
            }
            let old = app
            .current()
            .hovered()
            .map(|f| f.name.clone())
            .unwrap_or_default();
            match app.current_mut().rename_selected(&value) {
                Ok(()) => app.message(format!("renamed: {} → {}", old, value)),
                Err(e) => app.message(format!("rename failed: {}", e)),
            }
        }
        InputAction::Create => {
            let Some((is_dir, name)) = input::parse_create(&value) else {
                app.message("create cancelled".to_string());
                return Ok(());
            };
            if is_dir {
                match app.current_mut().create_dir(&name) {
                    Ok(()) => app.message(format!("created dir: {}", name)),
                    Err(e) => app.message(format!("mkdir failed: {}", e)),
                }
            } else {
                match app.current_mut().create_file(&name) {
                    Ok(()) => app.message(format!("created file: {}", name)),
                    Err(e) => app.message(format!("create failed: {}", e)),
                }
            }
        }
        InputAction::Search => { /* live-applied */ }
        InputAction::JumpToChar => {
            if let Some(c) = value.chars().next() {
                let _ = app.current_mut().jump_to_char(c);
            }
        }
        InputAction::Archive => {
            let name = value.trim().to_string();
            if name.is_empty() {
                app.message("archive cancelled".to_string());
                return Ok(());
            }
            let targets = app.targets();
            if targets.is_empty() {
                app.message("nothing to archive".to_string());
                return Ok(());
            }
            let cwd = app.current().path.clone();
            // Confirm with size estimate
            let size = archive::estimate_size(&targets);
            let msg = format!(
                "Archive {} item(s) ({}) → {}? (y/N)",
                              targets.len(),
                              preview::format_size(size),
                              name
            );
            app.confirm = Some(crate::app::Confirm {
                kind: crate::app::ConfirmKind::Archive { name, cwd },
                message: msg,
                targets,
            });
        }
        InputAction::Shell => {
            let Some(cmd) = shell::parse(&value) else {
                app.message("empty command".to_string());
                return Ok(());
            };
            let hovered = app.current().hovered().map(|e| e.path.clone());
            let selected: Vec<_> = app.selected.iter().cloned().collect();
            let cwd = app.current().path.clone();
            let expanded = shell::expand(&cmd.raw, hovered.as_deref(), &selected, &cwd);
            let expanded_cmd = shell::ShellCmd {
                raw: expanded.clone(),
                background: cmd.background,
            };

            if cmd.background {
                match shell::run(&expanded_cmd, &cwd) {
                    Ok(()) => app.message(format!("bg: {}", expanded)),
                    Err(e) => app.message(format!("shell failed: {}", e)),
                }
            } else if tmux::in_tmux() {
                // In tmux, run in a new pane so machina stays visible
                let args = ["sh", "-c", &expanded];
                match tmux::split(&cwd, &args, true) {
                    Ok(()) => app.message(format!("tmux: {}", expanded)),
                    Err(e) => app.message(format!("tmux failed: {}", e)),
                }
            } else {
                // Foreground: suspend and run
                // Note: needs &mut Term; handled in caller. Skip here.
                app.message(format!("running: {}", expanded));
                app.pending_fg_shell = Some(expanded_cmd);
            }
        }
        InputAction::Teleport => {
            let path_str = value.trim();
            if path_str.is_empty() {
                app.message("teleport cancelled".to_string());
                return Ok(());
            }
            let expanded = if path_str.starts_with('~') {
                if let Ok(home) = std::env::var("HOME") {
                    path_str.replacen("~", &home, 1)
                } else {
                    path_str.to_string()
                }
            } else {
                path_str.to_string()
            };
            let path = std::path::PathBuf::from(&expanded);
            if !path.exists() {
                app.message(format!(
                    "os error: no file or directory found (os error 2)"
                ));
                return Ok(());
            }
            if !path.is_dir() {
                app.message(format!("os error: not a directory (os error 20)"));
                return Ok(());
            }
            match app.current_mut().load_path(&path) {
                Ok(()) => app.message(format!("teleported to: {}", path.display())),
                Err(e) => app.message(format!("teleport failed: {}", e)),
            }
        }
    }
    Ok(())
}

fn handle_normal(app: &mut App, key: KeyEvent, term: &mut Term) -> Result<bool> {
    let pending = app.keybind.get_pending();

    // === Chord completions ===
    if let Some(p) = pending {
        match (p, key.code) {
            // gg = top of list
            ('g', KeyCode::Char('g')) => {
                app.current_mut().cursor_top();
                app.keybind.clear_pending();
                return Ok(true);
            }
            // gt = next tab, gT = prev tab
            ('g', KeyCode::Char('t')) => {
                app.next_tab();
                app.keybind.clear_pending();
                return Ok(true);
            }
            ('g', KeyCode::Char('T')) => {
                app.prev_tab();
                app.keybind.clear_pending();
                return Ok(true);
            }
            // yy / yp / yn / yd
            ('y', KeyCode::Char('y')) => {
                ops::yank_targets(app)?;
                app.keybind.clear_pending();
                return Ok(true);
            }
            ('y', KeyCode::Char('p')) => {
                if let Some(e) = app.current().hovered() {
                    let s = e.path.display().to_string();
                    match clipboard::copy(&s) {
                        Ok(()) => app.message(format!("clipboard: {}", s)),
                        Err(e) => app.message(format!("clipboard failed: {}", e)),
                    }
                }
                app.keybind.clear_pending();
                return Ok(true);
            }
            ('y', KeyCode::Char('n')) => {
                if let Some(e) = app.current().hovered() {
                    match clipboard::copy(&e.name) {
                        Ok(()) => app.message(format!("clipboard: {}", e.name)),
                        Err(e) => app.message(format!("clipboard failed: {}", e)),
                    }
                }
                app.keybind.clear_pending();
                return Ok(true);
            }
            ('y', KeyCode::Char('d')) => {
                let path = app.current().path.display().to_string();
                match clipboard::copy(&path) {
                    Ok(()) => app.message(format!("clipboard: {}", path)),
                    Err(e) => app.message(format!("clipboard failed: {}", e)),
                }
                app.keybind.clear_pending();
                return Ok(true);
            }
            // dd = cut
            ('d', KeyCode::Char('d')) => {
                ops::cut_targets(app)?;
                app.keybind.clear_pending();
                return Ok(true);
            }
            // g<key> bookmark
            ('g', KeyCode::Char(c)) => {
                let key_str = c.to_string();
                if let Some(target) = app.config.bookmarks.get(&key_str).cloned() {
                    if target.exists() {
                        app.current_mut().path = target;
                        app.current_mut().load()?;
                    } else {
                        app.message(format!("bookmark {} not found", key_str));
                    }
                    app.keybind.clear_pending();
                    return Ok(true);
                }
                app.keybind.clear_pending();
            }
            _ => app.keybind.clear_pending(),
        }
    }

    // === Single-press actions ===
    match (key.code, key.modifiers) {
        (KeyCode::Char('q'), _) => return Ok(false),

        // Up directory
        (KeyCode::Char('h'), _)
        | (KeyCode::Left, _)
        | (KeyCode::Esc, _)
        | (KeyCode::Char('H'), _) => {
            if !app.selected.is_empty() {
                // Esc clears selection first if any
                app.clear_selected();
            } else {
                app.current_mut().parent()?;
            }
        }

        // Down/Up
        (KeyCode::Char('j'), _) | (KeyCode::Down, _) => app.current_mut().cursor_down(40),
        (KeyCode::Char('k'), _) | (KeyCode::Up, _) => app.current_mut().cursor_up(),

        // Open: l, Right, Enter
        (KeyCode::Char('l'), _) | (KeyCode::Right, _) | (KeyCode::Enter, _) => {
            if !app.current_mut().enter_dir()? {
                open_hovered(app, term)?;
            }
        }

        // Page
        (KeyCode::PageDown, _) => app.current_mut().page_down(40),
        (KeyCode::PageUp, _) => app.current_mut().page_up(40),
        (KeyCode::Char('d'), KeyModifiers::CONTROL) => app.current_mut().page_down(40),
        (KeyCode::Char('u'), KeyModifiers::CONTROL) => app.current_mut().page_up(40),

        // Chord starters
        (KeyCode::Char('g'), m) if m.is_empty() => app.keybind.set_pending('g'),
        (KeyCode::Char('y'), m) if m.is_empty() => app.keybind.set_pending('y'),
        (KeyCode::Char('d'), m) if m.is_empty() => app.keybind.set_pending('d'),

        // Jump bottom
        (KeyCode::Char('G'), _) => app.current_mut().cursor_bottom(),

        // Paste with dialog
        (KeyCode::Char('p'), _) => ops::open_paste_dialog(app),

        // Delete: x = trash, D = permanent
        (KeyCode::Char('x'), _) => ops::trash_targets(app)?,
        (KeyCode::Char('D'), _) => ops::delete_targets_permanent(app)?,

        // Rename
        (KeyCode::Char('r'), _) => {
            if let Some(entry) = app.current().hovered() {
                let name = entry.name.clone();
                app.input.open(InputAction::Rename, "rename: ".into(), name);
            }
        }
        (KeyCode::Char('A'), _) => {
            if let Some(entry) = app.current().hovered() {
                let name = entry.name.clone();
                app.input
                .open_before_ext(InputAction::Rename, "rename: ".into(), name);
            }
        }

        // Create
        (KeyCode::Char('a'), mods) if mods.is_empty() => {
            app.input.open(
                InputAction::Create,
                "new (f=name / d=name): ".into(),
                           String::new(),
            );
        }

        // Search
        (KeyCode::Char('/'), _) => {
            app.input
            .open(InputAction::Search, "search: ".into(), String::new());
        }

        // f<char> jump
        (KeyCode::Char('f'), _) => {
            app.input
            .open(InputAction::JumpToChar, "jump to: ".into(), String::new());
        }

        // Teleport (goto path)
        (KeyCode::Char('T'), _) => {
            app.input
            .open(InputAction::Teleport, "teleport: ".into(), String::new());
        }

        // Archive (compress to .tar.gz)
        (KeyCode::Char('z'), _) => {
            let targets = app.targets();
            if targets.is_empty() {
                app.message("nothing to archive".into());
            } else {
                let default = crate::archive::default_name(&targets);
                app.input.open(InputAction::Archive, "archive: ".into(), default);
            }
        }

        // Custom shell command
        (KeyCode::Char('!'), _) => {
            app.input.open(
                InputAction::Shell,
                "$ (use $f $@ $d ; trailing & = bg): ".into(),
                           String::new(),
            );
        }

        // Send selection to other pane (split mode)
        (KeyCode::Char('>'), _) => {
            ops::send_to_other_pane(app);
        }

        // Hidden / reload
        (KeyCode::Char('.'), _) => {
            app.current_mut().toggle_hidden();
            app.message(format!(
                "hidden: {}",
                if app.current().show_hidden {
                    "on"
                } else {
                    "off"
                }
            ));
        }
        (KeyCode::Char('R'), _) => {
            app.current_mut().load()?;
            app.message("reloaded".into());
        }

        // Visual & multi-select
        (KeyCode::Char('v'), _) | (KeyCode::Char('V'), _) => {
            app.keybind.enter_visual(app.current().cursor);
        }
        (KeyCode::Char('s'), _) => {
            // Toggle selection on hovered
            if let Some(e) = app.current().hovered() {
                let p = e.path.clone();
                app.toggle_selected(p);
            }
            app.current_mut().cursor_down(40);
        }
        (KeyCode::Char('a'), KeyModifiers::CONTROL) => {
            app.select_all();
            app.message(format!("selected {} item(s)", app.selected.len()));
        }
        (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
            app.clear_selected();
            app.message("cleared selection".into());
        }

        // Tabs
        (KeyCode::Char('t'), _) => {
            let p = app.current().path.clone();
            app.new_tab(p)?;
        }
        (KeyCode::Char('w'), KeyModifiers::CONTROL) => {
            app.close_tab();
        }

        // Split mode
        (KeyCode::Char('|'), _) => {
            app.toggle_split()?;
        }
        (KeyCode::Tab, _) => {
            app.swap_split_focus();
        }

        // Preview toggle
        (KeyCode::Char('P'), _) => {
            app.preview_visible = !app.preview_visible;
        }

        // Open in editor (tmux-aware: split pane if in tmux)
        (KeyCode::Char(' '), _) => {
            if let Some(entry) = app.current().hovered() {
                let path = entry.path.clone();
                let cmd = app.config.editor.clone();
                let cwd = app.current().path.clone();
                if tmux::in_tmux() {
                    let path_str = path.to_string_lossy().to_string();
                    let args = [cmd.as_str(), path_str.as_str()];
                    match tmux::split(&cwd, &args, true) {
                        Ok(()) => app.message(format!("tmux: {}", cmd)),
                        Err(e) => app.message(format!("tmux failed: {}", e)),
                    }
                } else {
                    suspend_and_run(term, || opener::open_with(&cmd, &path))?;
                }
            }
        }

        // Open shell (tmux-aware: split pane if in tmux)
        (KeyCode::Char('S'), _) => {
            let cwd = app.current().path.clone();
            if tmux::in_tmux() {
                let sh = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
                let args = [sh.as_str()];
                match tmux::split(&cwd, &args, true) {
                    Ok(()) => app.message("tmux shell".into()),
                    Err(e) => app.message(format!("tmux failed: {}", e)),
                }
            } else {
                suspend_and_run(term, || opener::open_shell(&cwd))?;
            }
        }

        // Help
        (KeyCode::Char('?'), _) => {
            app.help = HelpVisible::Shown;
        }

        _ => {}
    }

    Ok(true)
}

fn handle_visual(app: &mut App, key: KeyEvent, _term: &mut Term) -> Result<bool> {
    match (key.code, key.modifiers) {
        (KeyCode::Esc, _) | (KeyCode::Char('v'), _) | (KeyCode::Char('V'), _) => {
            app.keybind.exit_visual();
        }
        (KeyCode::Char('j'), _) | (KeyCode::Down, _) => {
            app.current_mut().cursor_down(40);
            let c = app.current().cursor;
            app.keybind.extend_visual(c);
        }
        (KeyCode::Char('k'), _) | (KeyCode::Up, _) => {
            app.current_mut().cursor_up();
            let c = app.current().cursor;
            app.keybind.extend_visual(c);
        }
        (KeyCode::PageDown, _) => {
            app.current_mut().page_down(40);
            let c = app.current().cursor;
            app.keybind.extend_visual(c);
        }
        (KeyCode::PageUp, _) => {
            app.current_mut().page_up(40);
            let c = app.current().cursor;
            app.keybind.extend_visual(c);
        }
        (KeyCode::Char('G'), _) => {
            app.current_mut().cursor_bottom();
            let c = app.current().cursor;
            app.keybind.extend_visual(c);
        }
        (KeyCode::Char('g'), _) => {
            app.current_mut().cursor_top();
            let c = app.current().cursor;
            app.keybind.extend_visual(c);
        }
        // Visual range -> selection set
        (KeyCode::Char('s'), _) => {
            if let Some((start, end)) = app.keybind.get_visual_range() {
                let end = end.min(app.current().files.len().saturating_sub(1));
                let paths: Vec<_> = app.current().files[start..=end]
                .iter()
                .map(|f| f.path.clone())
                .collect();
                for p in paths {
                    app.selected.insert(p);
                }
            }
            app.keybind.exit_visual();
            app.message(format!("selected {}", app.selected.len()));
        }
        // Operate on visual range directly
        (KeyCode::Char('y'), _) => {
            if let Some((start, end)) = app.keybind.get_visual_range() {
                let end = end.min(app.current().files.len().saturating_sub(1));
                let paths: Vec<_> = app.current().files[start..=end]
                .iter()
                .map(|f| f.path.clone())
                .collect();
                let n = paths.len();
                app.file_op = Some(app::FileOp {
                    mode: OpMode::Copy,
                    files: paths,
                });
                app.message(format!("yanked {} item(s)", n));
            }
            app.keybind.exit_visual();
        }
        (KeyCode::Char('d'), _) => {
            if let Some((start, end)) = app.keybind.get_visual_range() {
                let end = end.min(app.current().files.len().saturating_sub(1));
                let paths: Vec<_> = app.current().files[start..=end]
                .iter()
                .map(|f| f.path.clone())
                .collect();
                let n = paths.len();
                app.file_op = Some(app::FileOp {
                    mode: OpMode::Cut,
                    files: paths,
                });
                app.message(format!("cut {} item(s)", n));
            }
            app.keybind.exit_visual();
        }
        (KeyCode::Char('x'), _) => {
            if let Some((start, end)) = app.keybind.get_visual_range() {
                let end = end.min(app.current().files.len().saturating_sub(1));
                for p in app.current().files[start..=end]
                    .iter()
                    .map(|f| f.path.clone())
                    .collect::<Vec<_>>()
                    {
                        app.selected.insert(p);
                    }
            }
            app.keybind.exit_visual();
            ops::trash_targets(app)?;
        }
        (KeyCode::Char('D'), _) => {
            if let Some((start, end)) = app.keybind.get_visual_range() {
                let end = end.min(app.current().files.len().saturating_sub(1));
                for p in app.current().files[start..=end]
                    .iter()
                    .map(|f| f.path.clone())
                    .collect::<Vec<_>>()
                    {
                        app.selected.insert(p);
                    }
            }
            app.keybind.exit_visual();
            ops::delete_targets_permanent(app)?;
        }
        (KeyCode::Char('q'), _) => return Ok(false),
        _ => {}
    }
    Ok(true)
}

fn open_hovered(app: &mut App, term: &mut Term) -> Result<()> {
    let Some(entry) = app.current().hovered() else {
        return Ok(());
    };
    let path = entry.path.clone();
    let ext = path
    .extension()
    .and_then(|e| e.to_str())
    .map(str::to_lowercase);

    let cmd = ext
    .as_deref()
    .and_then(|e| app.config.openers.get(e).cloned())
    .unwrap_or_else(|| "xdg-open".to_string());

    if opener::is_terminal_app(&cmd) {
        suspend_and_run(term, || opener::open_with(&cmd, &path))?;
        app.message(format!("opened in {}", short_cmd(&cmd)));
    } else {
        match opener::open_with(&cmd, &path) {
            Ok(()) => app.message(format!("launched: {}", short_cmd(&cmd))),
            Err(e) => app.message(format!("open failed: {}", e)),
        }
    }
    Ok(())
}

fn short_cmd(cmd: &str) -> String {
    std::path::Path::new(cmd)
    .file_name()
    .and_then(|n| n.to_str())
    .map(|s| s.to_string())
    .unwrap_or_else(|| cmd.to_string())
}

fn suspend_and_run<F: FnOnce() -> Result<()>>(term: &mut Term, f: F) -> Result<()> {
    disable_raw_mode()?;
    execute!(
        term.backend_mut(),
             LeaveAlternateScreen,
             DisableMouseCapture
    )?;
    let result = f();
    enable_raw_mode()?;
    execute!(term.backend_mut(), EnterAlternateScreen, EnableMouseCapture)?;
    term.clear()?;
    result
}
