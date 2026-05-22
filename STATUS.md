# machina — Build Status

A vim-keybinded, lazy-loading, OneShot-themed TUI file manager built in a day.
Yazi's keymap, Dolphin's GUI sensibilities, syntect previews, KGP images,
parallel folder-size caching, trash browser, bulk rename via $EDITOR.

**~4000 LOC of Rust. Built in ~12 hours.**

---

## ✅ Shipped

### Core
- [x] Lazy directory load — handles `/` without scanning the universe
- [x] ratatui 0.27 + crossterm 0.28, alt-screen, mouse capture
- [x] tokio async runtime, mpsc-channelled enumeration
- [x] OneShot TWM theme (`#0a0010` / `#c792ea` / `#00e5c8`)
- [x] CD-on-exit via shell wrapper (`mc` function)
- [x] notify-based filesystem watcher (auto-refresh @ 250ms)
- [x] Yazi-compatible chord state machine (1s timeout)

### Navigation
- [x] hjkl / arrows / Enter / Esc / H
- [x] `gg` / `G` / Ctrl-d / Ctrl-u (page)
- [x] `f <c>` jump-to-char (case-insensitive, wraps)
- [x] `/` live filter
- [x] `.` toggle hidden, `R` reload
- [x] `b` teleport (path input with `~` expansion)
- [x] `F` find files (fd | fzf integration)

### File ops
- [x] `yy` yank, `dd` cut, `p` paste-dialog (Copy/Move/Link)
- [x] `x` trash with confirm, `D` permanent delete with confirm
- [x] `r` rename, `A` rename-before-ext (yazi)
- [x] `a` smart create (`f=name` / `d=name` / `name/`)
- [x] `M` bulk rename via $EDITOR (temp file diff)
- [x] `z` archive to .tar.gz with size estimate + confirm
- [x] `!` shell command with `$f $@ $d` template vars; trailing `&` = bg
- [x] Auto-rename on paste conflict (` (1)`, ` (2)`, ...)

### Selection
- [x] `s` persistent multi-select (yazi)
- [x] `v`/`V` visual mode with `j/k/G/gg/PgUp/PgDn`
- [x] Ctrl-a select all, Ctrl-c clear

### Tabs & Split
- [x] `t` new tab, Ctrl-w close, `gt`/`gT` cycle
- [x] `|` toggle split view, `Tab` swap active pane
- [x] `>` send selection to other pane (split mode)

### Bookmarks
- [x] `g <c>` jump to bookmark (config-driven)
- [x] `m <c>` set bookmark at runtime (writes to config.toml)
- [x] Default bookmarks pre-seeded per CLAUDE.md project map

### Sorting
- [x] `o n/s/m/e` sort by name / size / mtime / ext
- [x] `o r` reverse current sort
- [x] Press same key twice to toggle direction

### Folder size cache
- [x] `c s` compute folder size with jwalk (parallel)
- [x] Cached at `~/.cache/machina/sizes.json`
- [x] mtime-keyed invalidation
- [x] On-demand only (zero idle cost)

### Trash browser
- [x] `T` opens trash modal
- [x] `j/k` navigate, `s` multi-select, `g/G` top/bottom
- [x] Enter / `p` restore, `D` purge, `R` refresh
- [x] XDG trash spec (Linux) via `trash` crate `os_limited`

### Clipboard chords
- [x] `y p` copy file's path
- [x] `y n` copy file's name
- [x] `y d` copy current dirname
- [x] arboard, Wayland data-control

### Previews
- [x] Text with syntect (base16-mocha.dark)
- [x] Directory listing with count
- [x] Image: KGP overlay on kitty (PNG, JPG, GIF, WebP, BMP)
- [x] Binary detection (size only)

### UI polish
- [x] p10k-style git status segment (branch/+/~/?/↑↓), hover-aware
- [x] Which-key popup on chord pending (g/y/d/o/c/m)
- [x] Tab strip (only if >1 tab)
- [x] Paste dialog with Copy/Move/Link buttons
- [x] Confirm dialog (red for permanent, amber for trash, cyan for archive)
- [x] Help screen (`?`) with every keybind, sectioned
- [x] tmux integration: Space / S / `!` open in new pane if `$TMUX` set

### Config
- [x] TOML at `~/.config/machina/config.toml`
- [x] `[general]` show_hidden, confirm_delete, editor
- [x] `[bookmarks]` (g-key → path)
- [x] `[openers]` (ext → command)
- [x] `~/` expansion
- [x] First-run writes default config

---

## ✅ Shipped (round 2)

### Undo & history
- [x] `u` undo last rename/move (session stack of 32)
- [x] Move ops via Cut paste and rename auto-push to stack

### Disk usage view
- [x] `d u` ncdu-style bar chart of current dir
- [x] Sorted by size desc, % of total, horizontal bars
- [x] Enter on dir to navigate, on file to jump cursor

### Gitignore as hidden
- [x] Files matching `.gitignore` (or global excludes) are treated as **hidden**
- [x] `.` toggles them along with dotfiles
- [x] When shown, rendered in dim git_ignored color for differentiation
- [x] Disable per-folder via `[general] respect_gitignore = false`

### Permissions
- [x] Yazi-style perms (`-rwxr-xr-x`) shown in status bar for hovered
- [x] `+` opens chmod input bar; takes octal (`755`) or symbolic (`u+x`)
- [x] Applies to selection set if any, else hovered

### Symlinks
- [x] Display `name → target` in pane
- [x] Symlink-colored (default amber)
- [x] `g f` follows symlink to target's parent dir
- [x] is_dir detection follows links

### Theme from config
- [x] `[theme]` block in config.toml with hex overrides for every color
- [x] All UI reads from runtime `OnceLock<Theme>` (set at startup)
- [x] Defaults to OneShot TWM palette

### Archive preview
- [x] `.zip` (via `zip` crate) lists entries with file/dir icons
- [x] `.tar` / `.tar.gz` (via `tar` + `flate2`) lists entries
- [x] Shows total count + file size header

### Icons (4 modes via `icons = "..."` in config)
- [x] `"nerd"` — Nerd-Font PUA glyphs (default; requires Nerd-Font kitty config)
- [x] `"image"` — kitty graphics protocol PNG sprites (U+10EEEE placeholders, not widely supported)
- [x] `"ascii"` — plain text marker (`>` dirs, `@` symlinks)
- [x] `"off"` — no icon column at all
- [x] Per-extension icons: Rust, C, Python, JS, TS, Go, Zig, Lua…
- [x] Per-directory icons: Downloads, Documents, Pictures, Videos, Music, Projects, .config, .git, target, node_modules
- [x] Special filenames: Cargo.toml, package.json, README, LICENSE, Makefile, Dockerfile, .gitignore, .env
- [x] Symlink arrow ()
- [x] Categories: archives, images, audio, video, fonts, docs, executables
- [x] Sprite generator: `tools/gen_icons.py` (PIL) → `assets/icons/*.png` (56 sprites, baked in)
- [x] Image mode auto-degrades to `"nerd"` when not in kitty or kitty < 1.24

### Mouse breadcrumbs
- [x] Click any segment of the path in header → jump to that path
- [x] Uses already-enabled mouse capture

---

## ⏳ Backlog

### Medium
- [ ] **Per-tab cursor memory** — restore cursor when switching tabs
- [ ] **Folder color rules** — match by name pattern (LS_COLORS-style)
- [ ] **Mouse drag-select** — visual range via mouse
- [ ] **Trash sort/filter** — sort by deleted time, filter by name
- [ ] **Color-only icons toggle** — disable icons for non-Nerd-Font terminals

---

## 🚀 Use

```bash
cd ~/machina
cargo build --release
cp target/release/machina ~/.local/bin/
source machina.sh           # adds `mc` shell function for cd-on-exit
mc
```

Press `?` once running for full keybinds.
