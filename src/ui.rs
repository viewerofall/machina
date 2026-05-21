use crate::app::{App, Folder, HelpVisible};
use crate::preview::{self, Preview};
use crate::theme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

pub fn draw(f: &mut Frame, app: &App) -> Option<Rect> {
    let area = f.size();
    let bg = Block::default().style(Style::default().bg(theme::BG).fg(theme::FG));
    f.render_widget(bg, area);

    // Tab strip only if more than one tab
    let has_tab_strip = app.tabs.len() > 1;
    let mut constraints = vec![Constraint::Length(1)]; // header
    if has_tab_strip {
        constraints.push(Constraint::Length(1)); // tabs
    }
    constraints.push(Constraint::Min(4)); // body
    constraints.push(Constraint::Length(1)); // input bar
    constraints.push(Constraint::Length(1)); // status

    let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints(constraints)
    .split(area);

    let mut i = 0usize;
    draw_header(f, app, chunks[i]);
    i += 1;
    if has_tab_strip {
        draw_tab_strip(f, app, chunks[i]);
        i += 1;
    }
    let body = chunks[i];
    i += 1;
    let input_row = chunks[i];
    i += 1;
    let status_row = chunks[i];

    let image_area = draw_body(f, app, body);

    draw_input(f, app, input_row);
    draw_status(f, app, status_row);

    if app.confirm.is_some() {
        draw_confirm(f, app, area);
    }
    if app.paste_dialog.is_some() {
        draw_paste_dialog(f, app, area);
    }
    if let Some(c) = app.keybind.get_pending() {
        draw_which_key(f, app, area, c);
    }
    if app.help == HelpVisible::Shown {
        draw_help(f, app, area);
    }

    image_area
}

fn draw_body(f: &mut Frame, app: &App, area: Rect) -> Option<Rect> {
    let split = app.split.is_some();
    let preview = app.preview_visible && !split;

    if split {
        // Two panes side-by-side. No preview while split.
        let parts = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

        let left_active = app.split == Some(app.tabs.len() - 1) || app.active != app.split.unwrap();
        // active is on its side; we draw "active" + split partner
        let (left_idx, right_idx) = if app.active < app.split.unwrap() {
            (app.active, app.split.unwrap())
        } else {
            (app.split.unwrap(), app.active)
        };

        draw_pane(f, app, &app.tabs[left_idx], parts[0], app.active == left_idx);
        draw_pane(f, app, &app.tabs[right_idx], parts[1], app.active == right_idx);
        let _ = left_active;
        None
    } else if preview {
        let parts = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(area);
        draw_pane(f, app, app.current(), parts[0], true);
        draw_preview(f, app, parts[1])
    } else {
        draw_pane(f, app, app.current(), area, true);
        None
    }
}

fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let path = app.current().path.display().to_string();
    let mut spans = vec![
        Span::styled(
            " machina ",
            Style::default()
            .bg(theme::ACCENT)
            .fg(theme::BG)
            .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(
            path,
            Style::default().fg(theme::FG).add_modifier(Modifier::BOLD),
        ),
    ];
    if app.current().show_hidden {
        spans.push(Span::styled("  [hidden]", Style::default().fg(theme::DIM)));
    }
    if !app.current().filter.is_empty() {
        spans.push(Span::styled(
            format!("  /{}", app.current().filter),
                Style::default().fg(theme::ACCENT),
        ));
    }
    if !app.selected.is_empty() {
        spans.push(Span::styled(
            format!("  [{} selected]", app.selected.len()),
                Style::default().fg(theme::WARN_FG),
        ));
    }

    f.render_widget(Paragraph::new(Line::from(spans)), area);

    // Show git status: either current folder or hovered directory
    let hovered_git = app.current().hovered().and_then(|e| {
        if e.is_dir {
            crate::git::GitStatus::detect(&e.path)
        } else {
            None
        }
    });

    let git_to_show = hovered_git.as_ref().or(app.current().git.as_ref());

    if let Some(git) = git_to_show {
        let git_spans = git_segment(git);
        let line = Line::from(git_spans.clone());
        let width: u16 = git_spans
        .iter()
        .map(|s| s.content.chars().count() as u16)
        .sum();
        if width < area.width {
            let right = Rect {
                x: area.x + area.width - width - 1,
                y: area.y,
                width: width + 1,
                height: 1,
            };
            f.render_widget(Paragraph::new(line), right);
        }
    }
}

fn draw_tab_strip(f: &mut Frame, app: &App, area: Rect) {
    let mut spans: Vec<Span> = Vec::with_capacity(app.tabs.len() * 2);
    for (i, t) in app.tabs.iter().enumerate() {
        let name = t.path.file_name().and_then(|n| n.to_str()).unwrap_or("/");
        let style = if i == app.active {
            Style::default()
            .bg(theme::ACCENT)
            .fg(theme::BG)
            .add_modifier(Modifier::BOLD)
        } else if Some(i) == app.split {
            Style::default()
            .bg(theme::VISUAL_BG)
            .fg(theme::FG)
            .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme::DIM)
        };
        spans.push(Span::styled(format!(" {} {} ", i + 1, name), style));
        spans.push(Span::raw(" "));
    }
    f.render_widget(Paragraph::new(Line::from(spans)), area);
}

fn git_segment(git: &crate::git::GitStatus) -> Vec<Span<'static>> {
    let (bg, fg) = if git.clean && git.ahead == 0 && git.behind == 0 {
        (Color::Rgb(0x00, 0x80, 0x40), Color::Rgb(0x0a, 0x00, 0x10))
    } else if !git.clean {
        (Color::Rgb(0xd7, 0xaf, 0x00), Color::Rgb(0x0a, 0x00, 0x10))
    } else {
        (theme::ACCENT, theme::BG)
    };

    let mut text = format!(" {} {}", "", git.branch);
    if git.ahead > 0 {
        text.push_str(&format!(" ↑{}", git.ahead));
    }
    if git.behind > 0 {
        text.push_str(&format!(" ↓{}", git.behind));
    }
    if git.staged > 0 {
        text.push_str(&format!(" +{}", git.staged));
    }
    if git.modified > 0 {
        text.push_str(&format!(" ~{}", git.modified));
    }
    if git.untracked > 0 {
        text.push_str(&format!(" ?{}", git.untracked));
    }
    text.push(' ');

    vec![Span::styled(
        text,
        Style::default().bg(bg).fg(fg).add_modifier(Modifier::BOLD),
    )]
}

fn draw_pane(f: &mut Frame, app: &App, folder: &Folder, area: Rect, active: bool) {
    let cursor = folder.cursor;
    let visual_range = if active {
        app.keybind.get_visual_range()
    } else {
        None
    };

    let items: Vec<ListItem> = folder
    .visible_files(area.height)
    .enumerate()
    .map(|(disp, entry)| {
        let actual = folder.offset + disp;
        let icon = if entry.is_dir { "" } else { "" };
        let is_sel = active && actual == cursor;
        let is_vis = visual_range
        .map(|(s, e)| actual >= s && actual <= e)
        .unwrap_or(false);
        let is_multi = app.selected.contains(&entry.path);

        let size_str = if entry.is_dir {
            String::new()
        } else {
            preview::format_size(entry.size)
        };

        let name_color = if entry.is_dir {
            theme::DIR_FG
        } else {
            theme::FILE_FG
        };

        let base_style = if is_sel {
            Style::default()
            .bg(theme::ACCENT)
            .fg(theme::BG)
            .add_modifier(Modifier::BOLD)
        } else if is_multi {
            Style::default()
            .bg(theme::VISUAL_BG)
            .fg(theme::WARN_FG)
            .add_modifier(Modifier::BOLD)
        } else if is_vis {
            Style::default().bg(theme::VISUAL_BG).fg(theme::FG)
        } else {
            Style::default().fg(name_color)
        };

        let marker = if is_multi { "*" } else { " " };
        let name_width = area.width.saturating_sub(26) as usize;
        let truncated = if entry.name.chars().count() > name_width {
            let mut s: String =
            entry.name.chars().take(name_width.saturating_sub(1)).collect();
            s.push('…');
            s
        } else {
            entry.name.clone()
        };

        Line::from(vec![
            Span::styled(
                format!("{} {}  {:<width$}", marker, icon, truncated, width = name_width),
                    base_style,
            ),
            Span::styled(
                format!(" {:>9} ", size_str),
                    if is_sel {
                        base_style
                    } else {
                        Style::default().fg(theme::DIM)
                    },
            ),
            Span::styled(
                format!("{:>11}", entry.modified),
                    if is_sel {
                        base_style
                    } else {
                        Style::default().fg(theme::DIM)
                    },
            ),
        ])
    })
    .map(ListItem::new)
    .collect();

    let title = if active {
        match app.keybind.mode {
            crate::app::keybind::Mode::Visual => {
                format!(" {} files  -- VISUAL -- ", folder.files.len())
            }
            _ => format!(" {} files ", folder.files.len()),
        }
    } else {
        let name = folder
        .path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("/");
        format!(" {}: {} files ", name, folder.files.len())
    };

    let border_color = if active {
        theme::ACCENT
    } else {
        theme::DIM
    };
    let block = Block::default()
    .title(Span::styled(
        title,
        Style::default().fg(border_color).add_modifier(Modifier::BOLD),
    ))
    .borders(Borders::ALL)
    .border_style(Style::default().fg(border_color));

    f.render_widget(List::new(items).block(block), area);
}

fn draw_preview(f: &mut Frame, app: &App, area: Rect) -> Option<Rect> {
    let block = Block::default()
    .title(Span::styled(
        " preview ",
        Style::default()
        .fg(theme::ACCENT)
        .add_modifier(Modifier::BOLD),
    ))
    .borders(Borders::ALL)
    .border_style(Style::default().fg(theme::DIM));

    let entry = match app.current().hovered() {
        Some(e) => e,
        None => {
            f.render_widget(block, area);
            return None;
        }
    };

    let inner = Block::default().borders(Borders::ALL).inner(area);
    let content = preview::get_preview(&entry.path);

    match content {
        Preview::Text(lines) => {
            f.render_widget(Paragraph::new(lines).block(block).wrap(Wrap { trim: false }), area);
            None
        }
        Preview::Directory(items, total) => {
            let lines: Vec<Line> = items
            .into_iter()
            .map(|s| {
                let color = if s.starts_with("📁") {
                    theme::DIR_FG
                } else {
                    theme::FG
                };
                Line::from(Span::styled(s, Style::default().fg(color)))
            })
            .collect();
            let mut all = vec![Line::from(Span::styled(
                format!(" {} items total", total),
                    Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD),
            ))];
            all.push(Line::from(""));
            all.extend(lines);
            f.render_widget(Paragraph::new(all).block(block).wrap(Wrap { trim: false }), area);
            None
        }
        Preview::Image { width, height, size } => {
            let header = vec![
                Line::from(Span::styled(
                    format!(" {}×{}   {}", width, height, preview::format_size(size)),
                        Style::default().fg(theme::DIM),
                )),
                Line::from(""),
            ];
            f.render_widget(Paragraph::new(header).block(block), area);

            if inner.height > 3 {
                Some(Rect {
                    x: inner.x,
                    y: inner.y + 2,
                    width: inner.width,
                    height: inner.height.saturating_sub(2),
                })
            } else {
                None
            }
        }
        Preview::Binary { size } => {
            let text = vec![
                Line::from(Span::styled(
                    " BINARY",
                    Style::default()
                    .fg(theme::WARN_FG)
                    .add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    format!(" Size: {}", preview::format_size(size)),
                        Style::default().fg(theme::FG),
                )),
            ];
            f.render_widget(Paragraph::new(text).block(block), area);
            None
        }
        Preview::Empty => {
            f.render_widget(
                Paragraph::new(Span::styled(
                    " (empty)",
                                            Style::default().fg(theme::DIM),
                ))
                .block(block),
                            area,
            );
            None
        }
        Preview::Error(e) => {
            f.render_widget(
                Paragraph::new(Span::styled(
                    format!(" {}", e),
                        Style::default().fg(theme::ERROR_FG),
                ))
                .block(block),
                            area,
            );
            None
        }
    }
}

fn draw_input(f: &mut Frame, app: &App, area: Rect) {
    if !app.input.active {
        return;
    }
    let line = Line::from(vec![
        Span::styled(
            app.input.prompt.clone(),
                     Style::default()
                     .fg(theme::BG)
                     .bg(theme::ACCENT)
                     .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
                          Span::styled(app.input.buffer.clone(), Style::default().fg(theme::FG)),
                          Span::styled("▏", Style::default().fg(theme::ACCENT)),
    ]);
    f.render_widget(Paragraph::new(line), area);
}

fn draw_status(f: &mut Frame, app: &App, area: Rect) {
    let mut spans: Vec<Span> = Vec::new();

    if let Some(msg) = app.messages.back() {
        spans.push(Span::styled(
            format!(" {}", msg),
                Style::default().fg(theme::DIM),
        ));
    }

    if let Some(op) = &app.file_op {
        let label = match op.mode {
            crate::app::OpMode::Copy => "yank",
            crate::app::OpMode::Cut => "cut",
            crate::app::OpMode::Link => "link",
        };
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            format!("[{} {}]", label, op.files.len()),
                Style::default().fg(theme::ACCENT),
        ));
    }

    if let Some(c) = app.keybind.get_pending() {
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            format!("({})", c),
                Style::default().fg(theme::WARN_FG),
        ));
    }

    f.render_widget(Paragraph::new(Line::from(spans)), area);
}

fn draw_confirm(f: &mut Frame, app: &App, area: Rect) {
    let Some(c) = &app.confirm else { return };

    let w = (area.width.saturating_sub(20)).min(80).max(40);
    let h = 5u16;
    let x = area.x + (area.width - w) / 2;
    let y = area.y + (area.height - h) / 2;
    let popup = Rect {
        x,
        y,
        width: w,
        height: h,
    };
    f.render_widget(Clear, popup);

    let color = match &c.kind {
        crate::app::ConfirmKind::DeletePermanent => theme::ERROR_FG,
        crate::app::ConfirmKind::DeleteTrash => theme::WARN_FG,
        crate::app::ConfirmKind::Archive { .. } => theme::ACCENT,
    };
    let title = match &c.kind {
        crate::app::ConfirmKind::DeletePermanent => " ! PERMANENT DELETE ",
        crate::app::ConfirmKind::DeleteTrash => " trash ",
        crate::app::ConfirmKind::Archive { .. } => " archive ",
    };

    let block = Block::default()
    .title(Span::styled(title, Style::default().fg(color).add_modifier(Modifier::BOLD)))
    .borders(Borders::ALL)
    .border_style(Style::default().fg(color))
    .style(Style::default().bg(theme::BG));

    let body = vec![
        Line::from(""),
        Line::from(Span::styled(format!(" {}", c.message), Style::default().fg(theme::FG))),
        Line::from(""),
    ];
    f.render_widget(Paragraph::new(body).block(block).wrap(Wrap { trim: false }), popup);
}

fn draw_paste_dialog(f: &mut Frame, app: &App, area: Rect) {
    let Some(d) = &app.paste_dialog else { return };

    let w = 60u16.min(area.width.saturating_sub(4));
    let h = 7u16;
    let x = area.x + (area.width - w) / 2;
    let y = area.y + (area.height - h) / 2;
    let popup = Rect { x, y, width: w, height: h };
    f.render_widget(Clear, popup);

    let block = Block::default()
    .title(Span::styled(
        " paste ",
        Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD),
    ))
    .borders(Borders::ALL)
    .border_style(Style::default().fg(theme::ACCENT))
    .style(Style::default().bg(theme::BG));

    let opts = ["Copy", "Move", "Link"];
    let mut row: Vec<Span> = Vec::new();
    for (i, name) in opts.iter().enumerate() {
        let style = if i == d.selected {
            Style::default()
            .bg(theme::ACCENT)
            .fg(theme::BG)
            .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme::FG)
        };
        row.push(Span::styled(format!("  {}  ", name), style));
        row.push(Span::raw("   "));
    }

    let n_files = app
    .file_op
    .as_ref()
    .map(|o| o.files.len())
    .unwrap_or(0);
    let dest = d
    .dest_override
    .as_ref()
    .cloned()
    .unwrap_or_else(|| app.current().path.clone());
    let info = format!(" {} item(s) → {}", n_files, dest.display());

    let body = vec![
        Line::from(""),
        Line::from(Span::styled(info, Style::default().fg(theme::DIM))),
        Line::from(""),
        Line::from(row),
        Line::from(""),
    ];
    f.render_widget(Paragraph::new(body).block(block), popup);
}

fn draw_which_key(f: &mut Frame, app: &App, area: Rect, chord: char) {
    let hints: Vec<(String, String)> = match chord {
        'g' => {
            let mut v = vec![
                ("g".to_string(), "top of list".to_string()),
                ("t".to_string(), "next tab".to_string()),
                ("T".to_string(), "prev tab".to_string()),
            ];
            let mut keys: Vec<&String> = app.config.bookmarks.keys().collect();
            keys.sort();
            for k in keys {
                if let Some(p) = app.config.bookmarks.get(k) {
                    v.push((k.clone(), format!("→ {}", short_path(p))));
                }
            }
            v
        }
        'y' => vec![
            ("y".to_string(), "yank (copy) file".to_string()),
            ("p".to_string(), "copy path to clipboard".to_string()),
            ("n".to_string(), "copy filename to clipboard".to_string()),
            ("d".to_string(), "copy dirname to clipboard".to_string()),
        ],
        'd' => vec![("d".to_string(), "cut file".to_string())],
        _ => return,
    };

    if hints.is_empty() {
        return;
    }

    let key_label = match chord {
        'g' => "g — goto/tab",
        'y' => "y — yank/copy",
        'd' => "d — cut",
        _ => "?",
    };

    let max_key = hints.iter().map(|(k, _)| k.chars().count()).max().unwrap_or(1);
    let max_desc = hints
    .iter()
    .map(|(_, d)| d.chars().count())
    .max()
    .unwrap_or(10);
    let w = ((max_key + max_desc + 6) as u16).max(20).min(area.width.saturating_sub(4));
    let h = (hints.len() as u16 + 2).min(area.height.saturating_sub(4));

    let x = area.x + area.width.saturating_sub(w + 2);
    let y = area.y + area.height.saturating_sub(h + 3);
    let popup = Rect { x, y, width: w, height: h };

    f.render_widget(Clear, popup);
    let block = Block::default()
    .title(Span::styled(
        format!(" {} ", key_label),
            Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD),
    ))
    .borders(Borders::ALL)
    .border_style(Style::default().fg(theme::DIM))
    .style(Style::default().bg(theme::BG));

    let lines: Vec<Line> = hints
    .into_iter()
    .map(|(k, d)| {
        Line::from(vec![
            Span::styled(
                format!(" {:>width$} ", k, width = max_key),
                    Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!("  {}", d), Style::default().fg(theme::FG)),
        ])
    })
    .collect();
    f.render_widget(Paragraph::new(lines).block(block), popup);
}

fn draw_help(f: &mut Frame, _app: &App, area: Rect) {
    let w = (area.width.saturating_sub(4)).min(80);
    let h = (area.height.saturating_sub(4)).min(34);
    let x = area.x + (area.width - w) / 2;
    let y = area.y + (area.height - h) / 2;
    let popup = Rect { x, y, width: w, height: h };
    f.render_widget(Clear, popup);

    let block = Block::default()
    .title(Span::styled(
        " machina keybinds (? or Esc to close) ",
                        Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD),
    ))
    .borders(Borders::ALL)
    .border_style(Style::default().fg(theme::ACCENT))
    .style(Style::default().bg(theme::BG));

    let style_key = Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD);
    let style_dim = Style::default().fg(theme::DIM);
    let style_fg = Style::default().fg(theme::FG);
    let row = |k: &str, d: &str| {
        Line::from(vec![
            Span::styled(format!(" {:<14}", k), style_key),
                   Span::styled(d.to_string(), style_fg),
        ])
    };
    let sec = |s: &str| {
        Line::from(Span::styled(format!(" {}", s), style_dim.add_modifier(Modifier::BOLD)))
    };

    let lines = vec![
        sec("Navigation"),
        row("h/←/Esc/H", "parent dir (Esc clears selection first)"),
        row("j/k", "down / up"),
        row("l/→/Enter", "enter dir or open file"),
        row("g g", "top   |   G  bottom"),
        row("Ctrl-d/u", "page down / up"),
        row("f <c>", "jump to file starting with <c>"),
        row(". / R", "toggle hidden / reload"),
        row("/ ", "search/filter"),
        sec("File ops"),
        row("y y", "yank (copy)"),
        row("d d", "cut"),
        row("p", "paste (copy/move/link dialog)"),
        row("x", "trash (confirm)"),
        row("D", "PERMANENT delete (confirm)"),
        row("r / A", "rename / rename before ext"),
        row("a", "create (f=name / d=name / name)"),
        sec("Selection"),
        row("s", "toggle select on hovered file"),
        row("v / V", "visual mode (range select)"),
        row("Ctrl-a/Ctrl-c", "select all / clear selection"),
        sec("Tabs & Split"),
        row("t / Ctrl-w", "new tab / close tab"),
        row("g t / g T", "next / prev tab"),
        row("|", "toggle split view"),
        row("Tab", "swap active pane (in split)"),
        row(">", "send selection to other pane (split)"),
        sec("Misc"),
        row("z", "archive selection to .tar.gz (confirm)"),
        row("!", "shell cmd: $f $@ $d  /  trail & = bg"),
        row("y p/n/d", "copy path / filename / dirname"),
        row("Space", "open in editor (tmux: new pane)"),
        row("S", "open shell here (tmux: new pane)"),
        row("P", "toggle preview pane"),
        row("? / q", "this help / quit"),
    ];

    f.render_widget(Paragraph::new(lines).block(block).wrap(Wrap { trim: false }), popup);
}

fn short_path(p: &std::path::Path) -> String {
    let s = p.display().to_string();
    if let Some(home) = dirs::home_dir() {
        let h = home.display().to_string();
        if let Some(rest) = s.strip_prefix(&h) {
            return format!("~{}", rest);
        }
    }
    s
}
