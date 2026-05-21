# machina - vim-style TUI file manager

A fast, keyboard-first file manager inspired by **yazi** and **ranger**.

## Features

✅ **Lazy loading** — handles large directories without lag (solves the `/` problem)
✅ **Vim keybinds** — hjkl navigation, yazi-style operations (yy/dd/p/x)
✅ **Visual mode** — select multiple files (V key)
✅ **File operations** — copy, cut, paste, delete to trash
✅ **Preview pane** — toggle with space, shows text/directory contents
✅ **Messages** — operation log and status messages

## Usage

```bash
cargo build --release
./target/release/machina         # Start in current directory
./target/release/machina ~/pics  # Start in specific directory
```

## Keybinds

### Navigation
| Key | Action |
|-----|--------|
| `h` / `←` | Parent directory |
| `j` / `↓` | Down |
| `k` / `↑` | Up |
| `l` / `→` / `Enter` | Open file/directory |
| `gg` | Jump to top |
| `G` | Jump to bottom |
| `Page Down` | Page down |
| `Page Up` | Page up |

### File Operations (yazi-style)
| Key | Action |
|-----|--------|
| `yy` | Copy file |
| `dd` | Cut file |
| `p` | Paste |
| `x` | Delete (to trash) |
| `r` | Rename (planned) |

### Visual Mode
| Key | Action |
|-----|--------|
| `v` | Enter visual mode |
| `j`/`k` | Extend selection |
| `g`/`G` | Jump to top/bottom (extends selection) |
| `y` | Copy selected files |
| `d` | Cut selected files |
| `x` | Delete selected files |
| `Esc` / `v` | Exit visual mode |

### View
| Key | Action |
|-----|--------|
| `Space` | Toggle preview pane |
| `q` | Quit |

## Design

- **Yazi-inspired**: Lazy file loading via channels, vim keybinds, logical UI
- **Rust + ratatui**: Fast, responsive, zero external runtime
- **Minimal**: ~1000 lines of Rust, no complex abstractions
- **Personal**: Built for keyboard-first workflow on CachyOS

## TODO (Post-MVP)

- [ ] Search/filter (`/` key)
- [ ] Rename dialog (`r` key)
- [ ] Marks/bookmarks (`m` key)
- [ ] Tab support (`Ctrl-t`, `gt`)
- [ ] Batch rename
- [ ] Config file support
- [ ] Drag and drop
- [ ] Shell integration (`:shell`)

## Architecture

```
src/
├── main.rs       # Event loop, keybind dispatch
├── app.rs        # Application state
│   ├── folder.rs # File listing + cursor management
│   └── keybind.rs # Keybind state machine
├── ui.rs         # ratatui UI rendering
├── ops.rs        # File operations (copy/cut/paste/delete)
└── preview.rs    # Preview content generation
```

**Key insight**: Folder lazy-loads files directly (no recursion), enabling smooth navigation even in `/` or `~/.cargo`. All file ops async + cancellable.
