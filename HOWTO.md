# machina — HOWTO

The complete, exhaustive, morbidly-obese reference. Every keybind, every
config option, every quirk.

Built for [abyss](https://github.com/viewerofall). Inspired by yazi + Dolphin.
Vim keybinds, GUI sensibilities, OneShot TWM theme.

---

## Table of Contents

1. [Install](#install)
2. [CD-on-Exit](#cd-on-exit)
3. [Keybinds — Full Reference](#keybinds--full-reference)
   - [Navigation](#navigation)
   - [File Operations](#file-operations)
   - [Selection](#selection)
   - [Tabs & Split View](#tabs--split-view)
   - [Clipboard & Shell](#clipboard--shell)
   - [Archive & Custom Commands](#archive--custom-commands)
   - [View & Help](#view--help)
4. [Modal Dialogs](#modal-dialogs)
   - [Paste Dialog (Copy/Move/Link)](#paste-dialog-copymovelink)
   - [Confirm Dialog (y/N)](#confirm-dialog-yn)
   - [Input Bar](#input-bar)
   - [Help Screen](#help-screen)
   - [Which-Key Popup](#which-key-popup)
5. [Config — Every Setting](#config--every-setting)
   - [Location](#location)
   - [`[general]`](#general)
   - [`[bookmarks]`](#bookmarks)
   - [`[openers]`](#openers)
6. [Git Status Display](#git-status-display)
7. [Image Previews (Kitty)](#image-previews-kitty)
8. [tmux Integration](#tmux-integration)
9. [Custom Commands — Template Vars](#custom-commands--template-vars)
10. [Smart Create Syntax](#smart-create-syntax)
11. [The Multi-Select Model](#the-multi-select-model)
12. [Theming (OneShot TWM)](#theming-oneshot-twm)
13. [Architecture (How It Works)](#architecture-how-it-works)
14. [Troubleshooting](#troubleshooting)
15. [Cheat Sheet (Print This)](#cheat-sheet-print-this)

---

## Install

```bash
cd ~/machina
cargo build --release
mkdir -p ~/.local/bin
cp target/release/machina ~/.local/bin/
```

Make sure `~/.local/bin` is in your `$PATH`:

```bash
echo $PATH | tr ':' '\n' | grep .local/bin
```

If not, add to `~/.zshrc`:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

### First run

```bash
machina           # current directory
machina ~/woven   # specific dir
```

On first launch, `~/.config/machina/config.toml` is auto-generated if missing.

---

## CD-on-Exit

Source the wrapper script to get the `mc()` shell function:

```bash
# In ~/.zshrc
source ~/machina/machina.sh
```

Now use `mc` instead of `machina`:

```bash
mc              # opens machina in cwd
mc ~/woven      # opens at ~/woven
```

When you quit (`q`), your shell automatically `cd`s to wherever machina was last.

### How it works

The `mc()` function:
1. Creates a temp file
2. Sets `MACHINA_CWD_FILE=<tmp>` in the env
3. Runs `machina`
4. On exit, machina writes its final path to that file
5. The shell reads it and runs `builtin cd`

Plain `machina` (no wrapper) just doesn't write the file — nothing happens.

### Renaming the function

If `mc` conflicts (Midnight Commander, etc.), edit `~/machina/machina.sh`:

```bash
# Change `mc()` to whatever:
m() {  ...  }
fm() {  ...  }
y() {  ...  }
```

---

## Keybinds — Full Reference

### Navigation

| Key            | Action                                                          |
| -------------- | --------------------------------------------------------------- |
| `j` / `Down`   | Move cursor down                                                |
| `k` / `Up`     | Move cursor up                                                  |
| `h` / `Left`   | Go to parent directory                                          |
| `Esc`          | (1) Clear selection if any, *then* (2) parent directory         |
| `H`            | Parent directory (matches yazi)                                 |
| `l` / `Right`  | Enter dir if hovered is dir, else open file in configured app   |
| `Enter`        | Same as `l`                                                     |
| `g g`          | Jump to top of file list                                        |
| `G`            | Jump to bottom of file list                                     |
| `PgDn` / `Ctrl-d` | Page down                                                    |
| `PgUp` / `Ctrl-u` | Page up                                                      |
| `f`            | Quick jump: prompts for one character, jumps to next matching   |
| `g <bookmark>` | Jump to bookmark defined in config (e.g. `g h` → home)          |
| `m <c>`        | **Set bookmark** `<c>` to current dir (persists to config.toml) |
| `b`            | Teleport — input bar for arbitrary path (with `~` expansion)    |
| `F`            | Find files — runs `fd | fzf`, cd's to selection                 |
| `.`            | Toggle hidden files                                             |
| `R`            | Reload current directory                                        |

### File Operations

| Key       | Action                                                           |
| --------- | ---------------------------------------------------------------- |
| `y y`     | Yank (copy) selected/hovered file(s)                             |
| `d d`     | Cut selected/hovered file(s)                                     |
| `p`       | Paste — opens Copy/Move/Link dialog                              |
| `x`       | Move to **trash** (asks y/N confirmation)                        |
| `D`       | **PERMANENT delete** (asks y/N, red warning banner)              |
| `r`       | Rename — input bar with cursor at end                            |
| `A`       | Rename — input bar with cursor *before extension* (yazi-style)   |
| `a`       | Create new file or directory (see [Smart Create](#smart-create-syntax)) |
| `M`       | **Bulk rename** — opens `$EDITOR` with one name per line         |

#### Visual mode (range select)

| Key                     | Action                                |
| ----------------------- | ------------------------------------- |
| `v` or `V`              | Enter visual mode                     |
| `j` / `k`               | Extend selection                      |
| `g` / `G`               | Extend to top / bottom                |
| `y`                     | Yank visual range                     |
| `d`                     | Cut visual range                      |
| `x`                     | Trash visual range (confirm)          |
| `D`                     | Permanently delete visual range       |
| `s`                     | Add visual range to persistent select |
| `Esc` / `v`             | Exit visual mode                      |

### Selection

machina has *two* selection systems:

1. **Visual mode** — temporary range, like vim. Exits after one operation.
2. **Persistent multi-select** — survives navigation, even across tabs.

| Key       | Action                                                  |
| --------- | ------------------------------------------------------- |
| `s`       | Toggle select on hovered file, then cursor moves down   |
| `Ctrl-a`  | Select all files in current folder                      |
| `Ctrl-c`  | Clear persistent selection                              |
| `Esc`     | Clears selection if any (otherwise goes to parent dir)  |

Selected files show:
- A `*` marker before the icon
- A `[N selected]` chip in the header
- Highlighted background

**Any operation** (`yy`, `dd`, `x`, `D`, `p`, `z`, etc.) uses the selection set
if non-empty, else falls back to the hovered file.

### Tabs & Split View

| Key            | Action                                                          |
| -------------- | --------------------------------------------------------------- |
| `t`            | New tab at current path                                         |
| `Ctrl-w`       | Close current tab (last tab cannot be closed)                   |
| `g t`          | Cycle to next tab                                               |
| `g T`          | Cycle to previous tab                                           |
| `|` (pipe)     | Toggle split view — opens current path in second pane           |
| `Tab`          | Swap active pane (in split mode)                                |
| `>`            | Send selected/hovered files to *other* pane (opens paste dialog)|

In split mode:
- Active pane has a **cyan** border + bright title
- Inactive pane has a **dim** border + grayed title
- The preview pane is hidden automatically

### Clipboard & Shell

| Key       | Action                                                        |
| --------- | ------------------------------------------------------------- |
| `y p`     | Copy hovered file's **full path** to system clipboard         |
| `y n`     | Copy hovered file's **filename** to system clipboard          |
| `y d`     | Copy **current directory** path to system clipboard           |
| `Space`   | Open hovered file in `editor` (config setting)                |
| `S`       | Open a shell in current directory                             |

#### tmux-aware behavior

If `$TMUX` is set, `Space` and `S` open a **new tmux pane** instead of
suspending machina. So machina stays visible while you edit/shell next to it.

Without tmux: the alternate screen is suspended, editor takes over, restores on
exit.

### Archive & Custom Commands

| Key       | Action                                                           |
| --------- | ---------------------------------------------------------------- |
| `z`       | Archive selected/hovered into `.tar.gz` (input bar + confirm)    |
| `!`       | Run a shell command — supports template vars                     |

### Sorting

| Chord     | Action                                                          |
| --------- | --------------------------------------------------------------- |
| `o n`     | Sort by **name** (default)                                      |
| `o s`     | Sort by **size** (biggest first)                                |
| `o m`     | Sort by **mtime** (newest first)                                |
| `o e`     | Sort by **extension** (then name)                               |
| `o r`     | Reverse the **current** sort direction                          |

Pressing the same sort key twice toggles its direction. Directories always
sort first regardless of mode.

### Folder Size (Cached)

| Chord     | Action                                                          |
| --------- | --------------------------------------------------------------- |
| `c s`     | Compute size of hovered/selected dir(s) via `jwalk` (parallel)  |

The result is cached at `~/.cache/machina/sizes.json`, keyed by `(path, mtime)`.
Subsequent loads of the same dir instantly show the cached size in the size
column — no `du` re-scan. When the dir's mtime changes, the cache is
invalidated and recomputed on next `c s`.

Cost: zero. Nothing runs unless you ask. Combine with `o s` to surface the
biggest folders in a directory.

### Undo

| Key       | Action                                                          |
| --------- | --------------------------------------------------------------- |
| `u`       | Undo last rename or move (stack of 32, session-scoped)          |

Only `Cut`-paste moves and `r`/`A`/`M` renames are undoable. Trashed files
should be restored from `T` (trash browser). Copy and Link are non-destructive
so they don't need undo.

### Symlinks

Symlinks are shown as `name → target` in the file list. They get the
`symlink_fg` theme color (default amber).

| Chord     | Action                                                          |
| --------- | --------------------------------------------------------------- |
| `g f`     | Follow symlink: jump to its target directory                    |

### Permissions

The full Unix mode (`-rwxr-xr-x`) of the **hovered** file is shown at the start
of the bottom status bar, yazi-style.

| Key       | Action                                                          |
| --------- | --------------------------------------------------------------- |
| `+`       | Open chmod input bar (e.g. `755`, `u+x`, `g-w`)                 |

Applies to the selection set if any, else hovered. Uses the system `chmod`
binary, so any mode syntax that supports works.

### Disk Usage View

| Chord     | Action                                                          |
| --------- | --------------------------------------------------------------- |
| `d u`     | Open ncdu-style bar chart of current directory                  |

Inside the view:

| Key                | Action                                  |
| ------------------ | --------------------------------------- |
| `j` / `k`          | Move cursor                             |
| `g` / `G`          | Top / bottom                            |
| `Enter` / `l`      | Open dir / jump cursor to file          |
| `q` / `Esc`        | Close view                              |

Entries are sorted by size desc, with horizontal bars showing relative size
and `%` of folder total. Uses the same jwalk cache as `c s` — instant for
folders you've already sized.

### Mouse — Breadcrumbs

Click any path segment in the **header bar** to jump to that path. The
`machina` chip is fixed-width, so segments start at column 10. Mouse capture
is always on (already used for resize events).

### Trash Browser

| Key in trash modal | Action                                  |
| ------------------ | --------------------------------------- |
| `j` / `k`          | Move cursor                             |
| `g` / `G`          | Top / bottom                            |
| `s`                | Toggle multi-select on current row      |
| `Enter` / `p`      | **Restore** selected (or hovered)       |
| `D`                | **Purge** selected (gone forever)       |
| `R`                | Refresh listing                         |
| `T` / `q` / `Esc`  | Close trash view                        |

Opens with capital `T` from normal mode. Reads via the XDG trash spec, so it
sees anything sent to trash by Dolphin, Nautilus, machina's `x`, etc. Shows
deletion timestamp + original path.

### View & Help

| Key       | Action                                                          |
| --------- | --------------------------------------------------------------- |
| `P`       | Toggle preview pane                                             |
| `/`       | Open search/filter (live-filters the file list as you type)     |
| `?`       | Show full help screen popup                                     |
| `q`       | Quit                                                            |

---

## Modal Dialogs

machina has 5 modal states. Keys map differently in each.

### Paste Dialog (Copy/Move/Link)

Triggered by `p` (or `>` to send to other pane).

```
┌── paste ────────────────────────────────────┐
│                                             │
│  3 item(s) → /home/abyss/Documents          │
│                                             │
│  [ Copy ]    Move      Link                 │
│                                             │
└─────────────────────────────────────────────┘
```

| Key              | Action                                  |
| ---------------- | --------------------------------------- |
| `←` / `→`        | Select option                           |
| `h` / `l`        | Select option (vim-style)               |
| `c`              | Jump to Copy                            |
| `m`              | Jump to Move                            |
| `s`              | Jump to (Sym)Link                       |
| `Enter`          | Confirm                                 |
| `Esc` / `q`      | Cancel                                  |

The default mode is whichever you yanked with (`yy` → Copy, `dd` → Move).

### Confirm Dialog (y/N)

Triggered by `x` (trash), `D` (delete), `z` confirmation.

```
┌── ! PERMANENT DELETE ──────────────────────┐
│                                            │
│  PERMANENTLY delete: secret.txt? (y/N)     │
│                                            │
└────────────────────────────────────────────┘
```

| Key              | Action                                   |
| ---------------- | ---------------------------------------- |
| `y` / `Y`        | Confirm                                  |
| `n` / `N` / `Esc`| Cancel                                   |

Color codes:
- 🔴 Red — permanent delete
- 🟡 Yellow — trash
- 🔵 Cyan — archive

### Input Bar

Triggered by `r` (rename), `a` (create), `/` (search), `f` (jump), `z`
(archive), `!` (shell). Shows at the bottom of the screen.

```
 rename:  hello.md ▏
```

| Key              | Action                                |
| ---------------- | ------------------------------------- |
| Any char         | Insert                                |
| `Backspace`      | Delete char before cursor             |
| `Delete`         | Delete char under cursor              |
| `←` / `→`        | Move cursor                           |
| `Home` / `End`   | Jump to start / end                   |
| `Enter`          | Commit (run the action)               |
| `Esc`            | Cancel                                |

For search, results update live as you type.

### Help Screen

`?` opens a giant cheat sheet. `?` / `Esc` / `q` to close.

### Which-Key Popup

When you press a *chord starter* (`g`, `y`, `d`, `o`, `c`, or `m`), a popup
appears in the bottom-right showing what comes next. Bookmarks populate
dynamically from config when you press `g`.

Auto-dismisses after 1 second or on next key press.

### Bulk Rename Modal (via `$EDITOR`)

`M` writes one name per line to a temp file, opens it in your configured editor
(usually `lvim`/`nvim`). On save & quit, machina diffs the lines against the
originals and renames in order.

Rules:
- Line count must match exactly (1 line per file).
- Comment lines starting with `#` are ignored.
- A line that would contain `/` is skipped (no path moves — that's `p`'s job).
- A target name that already exists is skipped silently.

For a single file: this is a heavier rename than `r`. For 50 files: this is
the killer feature you came for.

---

## Config — Every Setting

### Location

```
~/.config/machina/config.toml
```

Auto-created on first run if missing. Use `R` inside machina to reload after
edits.

### `[general]`

```toml
[general]
show_hidden    = false                          # show . files by default
confirm_delete = true                           # show y/N for x and D
editor         = "/home/abyss/.local/bin/lvim"  # used by Space and Enter on text files
```

- `show_hidden` (bool, default `false`) — initial state of the hidden-files
  toggle. `.` key toggles at runtime.
- `confirm_delete` (bool, default `true`) — if false, `x` skips the y/N prompt
  for trash. `D` always asks regardless.
- `editor` (string, default `$EDITOR` or `nvim`) — invoked by `Space`. Can be
  any binary on `$PATH` or an absolute path.
- `respect_gitignore` (bool, default `true`) — when true, files matched by any
  `.gitignore` are treated as **hidden** (toggle with `.` like dotfiles).
- `icons` (string, default `"nerd"`) — icon rendering mode. One of:
  - `"nerd"` — Nerd-Font PUA glyphs. Requires a Nerd Font in your terminal,
    e.g. in kitty: `symbol_map U+E000-U+F8FF,U+F0000-U+FFFFF JetBrainsMono Nerd Font`
  - `"image"` — kitty graphics protocol with baked-in PNG sprites
    (`assets/icons/*.png`, generated by `tools/gen_icons.py`). Only works
    inside kitty; automatically falls back to `"nerd"` elsewhere.
  - `"ascii"` — plain text marker (`>` for dirs, `@` for symlinks). For
    low-end / non-Unicode terminals.
  - `"off"` — no icon column at all.

### `[theme]`

```toml
[theme]
bg              = "#0a0010"
fg              = "#c792ea"
accent          = "#00e5c8"
dim             = "#6c7086"
visual_bg       = "#1c1032"
dir_fg          = "#00e5c8"
file_fg         = "#c792ea"
error_fg        = "#ff5555"
warn_fg         = "#ffb86c"
git_ignored_fg  = "#464a60"
symlink_fg      = "#ffd170"
```

All keys are optional — omitted ones use the OneShot defaults. Use any
6-digit hex (`#RRGGBB`). Reload by restarting machina; live reload is not yet
supported.

### `[bookmarks]`

```toml
[bookmarks]
h = "~"
d = "~/Downloads"
D = "~/Documents"
c = "~/.config"
r = "/"
p = "~/Projects"
w = "~/woven"
v = "~/veil"
m = "~/machina"
```

Single-character keys. Press `g<key>` in machina to jump.

- Keys are case-sensitive (`d` ≠ `D`)
- Tildes expand to `$HOME`
- Missing/invalid paths show "bookmark X not found" but don't crash
- The which-key popup auto-lists everything you define here

### `[openers]`

```toml
[openers]
rs    = "/home/abyss/.local/bin/lvim"
png   = "imv"
mp4   = "mpv"
pdf   = "zathura"
```

- Lowercase extensions only
- Value can be any command on `$PATH` or absolute path
- Used when you press `Enter` on a file
- Unrecognized extensions fall back to `xdg-open`

**Terminal-aware**: if the configured command is `nvim`, `vim`, `vi`, `lvim`,
`emacs`, `nano`, `helix`, `hx`, `less`, or `more`, machina suspends its UI so
the editor takes over the terminal. For non-terminal apps (`imv`, `mpv`,
`zathura`), the command is spawned detached and machina stays visible.

In tmux, terminal apps open in a new tmux pane (machina stays in its pane).

---

## Git Status Display

If the current directory is inside a git repo, the right side of the header
shows a p10k-style segment:

```
                                      main +1 ~3 ?2
```

Colors:
- 🟢 **Green** — clean working tree, up to date with remote
- 🟡 **Yellow** — has staged / modified / untracked files
- 🔵 **Cyan** — clean but ahead/behind remote

Symbols:
- ` <name>` — current branch
- `↑N` — N commits ahead of remote
- `↓N` — N commits behind remote
- `+N` — N staged files
- `~N` — N modified files
- `?N` — N untracked files

The status runs `git status --porcelain --branch` on directory change only.
On large repos this can take ~100ms. Sub-100ms otherwise.

---

## Image Previews (Kitty)

Detection (in this order):
1. `$KITTY_WINDOW_ID` env var is set → Kitty
2. `$TERM` is `xterm-kitty` → Kitty

When detected, the preview pane uses the **Kitty Graphics Protocol** to render
actual images of `.png`, `.jpg`, `.jpeg`, `.gif`, `.webp`, `.bmp`, `.ico`.

Images are:
- Resized to fit the preview pane (preserves aspect ratio, never upscales)
- PNG-encoded, base64'd, chunked at 4096 bytes
- Cleared on navigation (no leftover images)

Other terminals will see image metadata instead (dimensions + size). Iterm2,
sixel, and chafa fallback aren't implemented — open the image with `Enter` to
view in `imv` or whatever is configured.

---

## tmux Integration

Detected via `$TMUX` env var.

When in tmux:

| Action            | Behavior                                                 |
| ----------------- | -------------------------------------------------------- |
| `Space` (editor)  | Opens editor in new tmux pane (split horizontally)       |
| `S` (shell)       | Opens shell in new tmux pane                             |
| `!` (foreground)  | Foreground shell commands open in a new tmux pane        |
| `!cmd &`          | Background commands still spawn detached, no pane        |

When *not* in tmux:
- Editor / shell / `!` foreground commands suspend machina's alt-screen,
  take over the terminal, then restore machina on exit.

---

## Custom Commands — Template Vars

`!` opens a shell input prompt. Type any shell command. Supports template
expansion:

| Token   | Expands to                                                |
| ------- | --------------------------------------------------------- |
| `$f`    | Full path of hovered file (shell-quoted)                  |
| `$F`    | Filename only (basename) of hovered file                  |
| `$d`    | Current directory (shell-quoted)                          |
| `$@`    | All selected files (space-separated, each quoted)         |
| `$$`    | Literal `$` (not yet — just escape with `\$`)             |

If the command ends with `&`, it runs **detached in background**, machina stays
focused. Otherwise it runs in **foreground** (suspends machina, or opens in
tmux pane).

### Examples

```bash
$ wc -l $@                   # count lines in selected files
$ git log -p -- $f           # show git history of hovered file
$ kitty &                    # spawn new kitty window
$ ffmpeg -i $f out.mp4 &     # background convert
$ chmod +x $@                # batch chmod selected
$ du -sh $@                  # disk usage of selection
$ ls -la $d                  # ls of current dir
$ mpv $f &                   # play a video in background
```

### Safety

Foreground commands DO suspend the UI. Background commands have stdin/stdout
piped to /dev/null. If a background command writes data you want, redirect to a
file: `command > /tmp/log &`

---

## Smart Create Syntax

When you press `a`, the input bar opens with prompt `new (f=name / d=name):`.
Type one of:

| Input            | Result                            |
| ---------------- | --------------------------------- |
| `hello.md`       | Creates **file** `hello.md`       |
| `f=hello.md`     | Creates **file** `hello.md`       |
| `d=newdir`       | Creates **directory** `newdir`    |
| `newdir/`        | Creates **directory** `newdir`    |
| `f=`             | Cancelled (empty)                 |
| `d=`             | Cancelled (empty)                 |

Default (no prefix, no trailing `/`) is **file**.

Empty input or just `f=`/`d=` cancels with no action.

---

## The Multi-Select Model

Reading this once will save you confusion later.

There are **three** ways to operate on multiple files:

### 1. Visual mode (`v` / `V`)

Like vim. Press `v`, move with `j`/`k` to extend, then:
- `y` / `d` / `x` / `D` — operate on the range, *exit visual mode*
- `s` — **add** the range to persistent selection, exit visual mode
- `Esc` / `v` — exit without doing anything

Visual mode is for one-shot operations on a contiguous range.

### 2. Persistent multi-select (`s`)

Press `s` on a file → it's added to the selection set + cursor advances.
Press `s` again on a selected file → removed.

Selection **survives**:
- Cursor movement
- Directory navigation
- Tab switches
- Split-pane switches

It does NOT survive:
- `Ctrl-c` (explicit clear)
- `Esc` when something is selected (clears, doesn't navigate up)
- Successful operations (paste, delete) clear the selection afterward
- `q` (quit)

Use this for cross-directory operations: navigate around, mark stuff with `s`,
then `yy` / `p` to operate on everything at once.

### 3. Hovered (no selection)

If nothing is selected, operations target the **currently hovered file**.

So the priority is: **selection** > **hovered**. Always.

---

## Theming (OneShot TWM)

Hardcoded for now (config-driven theme is on the TODO):

| Element            | Color      |
| ------------------ | ---------- |
| Background         | `#0a0010`  |
| Foreground (files) | `#c792ea`  |
| Accent (dirs)      | `#00e5c8`  |
| Dim (borders/info) | `#6c7086`  |
| Visual selection   | `#1c1032`  |
| Persistent select  | warning yellow |
| Error              | `#ff5555`  |
| Warning            | `#ffb86c`  |
| Git clean          | `#008040`  |
| Git dirty          | `#d7af00`  |

Selected file in the file list: cyan background, black text, bold.

---

## Architecture (How It Works)

```
src/
├── main.rs        Event loop, keybind dispatch, terminal setup
├── app.rs         App state: tabs, selection, dialogs, config, etc.
│   ├── folder.rs  File listing, cursor, scroll, sort, filter, jump
│   └── keybind.rs Mode + pending key state machine
├── ui.rs          ratatui rendering: panes, dialogs, popups
├── ops.rs         File operations (copy/cut/paste/delete/trash)
├── preview.rs     Preview content (text/syntax/dir/image metadata)
├── kgp.rs         Kitty Graphics Protocol — image rendering
├── opener.rs      File handlers (xdg-open + per-ext config)
├── config.rs      TOML config loader
├── theme.rs       OneShot color palette
├── input.rs       Input bar state + create-command parser
├── git.rs         Git status (porcelain v1 parser)
├── clipboard.rs   System clipboard wrapper (arboard)
├── cwd_writer.rs  Write final cwd to $MACHINA_CWD_FILE on exit
├── watcher.rs     notify-based file system watcher
├── archive.rs     tar.gz creation
├── shell.rs       $ command parser + template expansion
└── tmux.rs        tmux pane / window helpers
```

### Event Loop

1. Render frame with `terminal.draw()`
2. Overlay Kitty image (if applicable)
3. Poll keyboard for up to 100ms
4. If event: dispatch through modal stack:
   - Help screen?
   - Paste dialog?
   - Confirm dialog?
   - Input bar?
   - Else: normal/visual mode handler
5. If `pending_fg_shell` was set: suspend + run + restore
6. Refresh file watcher target dirs
7. Drain file watcher events; reload tabs if any
8. Loop

### Key Decision Tree

```
keypress
  ├── help is open?    → close on Esc/q/? else swallow
  ├── paste dialog?    → option keys / Enter / Esc
  ├── confirm?         → y / N / Esc
  ├── input bar?       → type / Enter / Esc / arrow keys
  └── otherwise        → normal/visual mode
```

### Lazy File Loading

Unlike the original design which planned for `read_dir` with metadata
pre-computed, we use synchronous `read_dir` with per-entry metadata. For very
large dirs (`/usr/share/icons` style), this takes ~50-200ms. Acceptable for
v1; can be made truly async later if it becomes a pain.

---

## Troubleshooting

### `gg` doesn't work

You're probably pressing too slowly. The chord timeout is 1 second. If you
take longer than that between the first `g` and the second, the first is
discarded. Press faster.

### Images don't show in Kitty

Check `$TERM` is `xterm-kitty` and `$KITTY_WINDOW_ID` is set:

```bash
echo $TERM
echo $KITTY_WINDOW_ID
```

Both should be non-empty. If you're in tmux, you'll need Kitty's tmux passthrough
enabled. (Or just use machina outside tmux for images.)

### `mc` says "command not found"

You haven't sourced the wrapper. Run:

```bash
source ~/machina/machina.sh
```

Or open a new terminal after adding the source line to `~/.zshrc`.

### `mc` runs but cd doesn't happen

Make sure the binary `machina` is on `$PATH`:

```bash
which machina   # should print a path
```

### Editor opens but the screen looks weird

machina's alternate screen is suspended when launching a terminal editor. If
the editor (e.g. an old `vim`) doesn't reset its own screen on exit, you may
need to manually press `R` in machina to redraw. Or just resize the terminal.

### Git status is missing on a git repo

Make sure `git` is in `$PATH`. machina shells out to `git status --porcelain
--branch`. If that command works in your terminal but the status doesn't show,
file an issue.

### Clipboard copy (`yp/yn/yd`) fails

You need `wl-clipboard` installed (Wayland) or `xclip`/`xsel` (X11). On
CachyOS:

```bash
sudo pacman -S wl-clipboard
```

### Archive `z` failed

`tar` must be on `$PATH`. Should be standard on any Linux. The archive command
is roughly:

```bash
tar -czf <name>.tar.gz <relative-paths>
```

If selected files span multiple parent directories, tar will warn or fail.
Either group your selection within one directory, or use a custom `!` command.

---

## Cheat Sheet (Print This)

```
NAVIGATION                    SELECTION
  h ← Esc H   parent dir        s         toggle select
  j k ↓ ↑     down/up            Ctrl-a    select all
  l → Enter   open / enter       Ctrl-c    clear selection
  gg G        top / bottom       v V       visual mode
  f<c>        jump to char       Esc       clear sel else parent
  Ctrl-d/u    page down/up
  /           search            FILE OPS
  .           hidden toggle       yy        yank (copy)
  R           reload              dd        cut
  g<bm>       bookmark            p         paste (C/M/L)
                                  x         trash (confirm)
TABS & SPLIT                      D         PERMANENT delete
  t           new tab             r         rename
  Ctrl-w      close tab           A         rename pre-ext
  gt gT       next/prev           a         create (f=/d=)
  |           split toggle        z         archive
  Tab         swap pane
  >           send to pane      CLIPBOARD & SHELL
                                  yp        copy path
VIEW                              yn        copy filename
  P           preview toggle      yd        copy dirname
  ?           help                Space     editor (tmux: pane)
  q           quit                S         shell (tmux: pane)
                                  !         custom command
PASTE DIALOG                                 ($f $@ $d, & = bg)
  ← → h l     pick mode
  c m s       jump to mode
  Enter       commit
  Esc         cancel
```

---

*v0.1 — Built April 2026. Patches welcome (when there's a remote). For now,
`r` it and ship.*
