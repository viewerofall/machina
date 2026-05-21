#!/usr/bin/env python3
"""
Bake icon sprites for machina.

Generates one 32x32 PNG per icon kind into assets/icons/.
Each PNG is a simple geometric/colored mark (machina is a TUI; we just need
distinguishable glyphs the kitty graphics protocol can show in a single cell).

Run:  python3 tools/gen_icons.py
"""
from __future__ import annotations

import os
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

# ---------------------------------------------------------------------------
# Setup
# ---------------------------------------------------------------------------
SIZE = 32  # 32x32 sprite, kitty scales to one cell
ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "assets" / "icons"
OUT.mkdir(parents=True, exist_ok=True)

# Try to find a sane font; fall back to default if not present.
FONT_CANDIDATES = [
    "/usr/share/fonts/TTF/JetBrainsMonoNerdFont-Bold.ttf",
    "/usr/share/fonts/jetbrains-mono/JetBrainsMono-Bold.ttf",
    "/usr/share/fonts/TTF/JetBrainsMono-Bold.ttf",
    "/usr/share/fonts/TTF/DejaVuSans-Bold.ttf",
    "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
]
def load_font(sz):
    for p in FONT_CANDIDATES:
        if os.path.exists(p):
            return ImageFont.truetype(p, sz)
    return ImageFont.load_default()

FONT_BIG = load_font(22)
FONT_MED = load_font(16)
FONT_SMALL = load_font(11)

# ---------------------------------------------------------------------------
# Primitive drawing helpers
# ---------------------------------------------------------------------------
def new_canvas():
    return Image.new("RGBA", (SIZE, SIZE), (0, 0, 0, 0))

def center_text(draw, text, color, font=FONT_BIG):
    bbox = draw.textbbox((0, 0), text, font=font)
    w, h = bbox[2] - bbox[0], bbox[3] - bbox[1]
    x = (SIZE - w) // 2 - bbox[0]
    y = (SIZE - h) // 2 - bbox[1]
    draw.text((x, y), text, fill=color, font=font)

def rounded(draw, color, pad=2, radius=5):
    draw.rounded_rectangle(
        (pad, pad, SIZE - pad - 1, SIZE - pad - 1),
        radius=radius,
        fill=color,
    )

def badge(name, bg, fg, text, font=FONT_BIG):
    """Rounded square with a centered short label."""
    img = new_canvas()
    d = ImageDraw.Draw(img)
    rounded(d, bg)
    bbox = d.textbbox((0, 0), text, font=font)
    w, h = bbox[2] - bbox[0], bbox[3] - bbox[1]
    x = (SIZE - w) // 2 - bbox[0]
    y = (SIZE - h) // 2 - bbox[1]
    d.text((x, y), text, fill=fg, font=font)
    img.save(OUT / f"{name}.png")

def folder_icon(name, color=(0, 229, 200, 255), accent=None):
    """Draw a yazi-ish folder glyph."""
    img = new_canvas()
    d = ImageDraw.Draw(img)
    # tab
    d.rounded_rectangle((3, 6, 14, 11), radius=2, fill=color)
    # body
    d.rounded_rectangle((3, 9, 29, 27), radius=3, fill=color)
    if accent:
        # accent stripe inside body
        d.rectangle((6, 14, 26, 17), fill=accent)
    img.save(OUT / f"{name}.png")

def file_icon(name, color=(199, 146, 234, 255), label=None, label_color=(10, 0, 16, 255)):
    """Page-with-folded-corner glyph."""
    img = new_canvas()
    d = ImageDraw.Draw(img)
    # page body
    d.polygon(
        [(6, 3), (22, 3), (28, 9), (28, 28), (6, 28)],
        fill=color,
    )
    # folded corner
    d.polygon([(22, 3), (28, 9), (22, 9)], fill=(0, 0, 0, 100))
    if label:
        bbox = d.textbbox((0, 0), label, font=FONT_SMALL)
        w, h = bbox[2] - bbox[0], bbox[3] - bbox[1]
        x = (SIZE - w) // 2 - bbox[0]
        y = (SIZE - h) // 2 - bbox[1] + 3
        d.text((x, y), label, fill=label_color, font=FONT_SMALL)
    img.save(OUT / f"{name}.png")

# ---------------------------------------------------------------------------
# Sprite definitions
# ---------------------------------------------------------------------------
# Each badge: name, bg, fg, text
BADGES = [
    # languages
    ("rs",     (222, 165, 132, 255), (10, 0, 16, 255),     "R"),
    ("c",      (85, 130, 200, 255),  (255, 255, 255, 255), "C"),
    ("cpp",    (0, 89, 156, 255),    (255, 255, 255, 255), "C+"),
    ("py",     (255, 212, 59, 255),  (53, 114, 165, 255),  "Py"),
    ("js",     (247, 223, 30, 255),  (50, 50, 50, 255),    "JS"),
    ("ts",     (49, 120, 198, 255),  (255, 255, 255, 255), "TS"),
    ("html",   (228, 77, 38, 255),   (255, 255, 255, 255), "<>"),
    ("css",    (38, 77, 228, 255),   (255, 255, 255, 255), "#"),
    ("lua",    (0, 0, 128, 255),     (255, 255, 255, 255), "Lu"),
    ("go",     (0, 173, 216, 255),   (255, 255, 255, 255), "Go"),
    ("zig",    (247, 164, 29, 255),  (10, 0, 16, 255),     "Z"),
    ("java",   (244, 67, 54, 255),   (255, 255, 255, 255), "J"),
    ("rb",     (204, 52, 45, 255),   (255, 255, 255, 255), "Rb"),
    ("php",    (119, 123, 180, 255), (255, 255, 255, 255), "Ph"),
    ("sh",     (89, 89, 89, 255),    (199, 146, 234, 255), "$_"),
    ("md",     (60, 60, 80, 255),    (199, 146, 234, 255), "M↓"),
    ("toml",   (156, 66, 33, 255),   (255, 255, 255, 255), "T"),
    ("json",   (251, 192, 45, 255),  (50, 50, 50, 255),    "{}"),
    ("yaml",   (203, 23, 30, 255),   (255, 255, 255, 255), "Y"),
    ("xml",    (255, 152, 0, 255),   (10, 0, 16, 255),     "X"),
    ("txt",    (96, 96, 96, 255),    (255, 255, 255, 255), "T"),
    ("log",    (76, 96, 76, 255),    (200, 230, 200, 255), "L"),
    ("conf",   (76, 76, 96, 255),    (200, 200, 230, 255), "⚙"),
    # archive
    ("archive",(160, 120, 60, 255),  (255, 255, 255, 255), "Z"),
    # binaries / packages
    ("exe",    (140, 50, 50, 255),   (255, 255, 255, 255), "▶"),
    ("iso",    (60, 60, 60, 255),    (200, 200, 200, 255), "◉"),
    ("font",   (180, 100, 200, 255), (255, 255, 255, 255), "Aa"),
    ("pdf",    (220, 40, 40, 255),   (255, 255, 255, 255), "PDF"),
    # lock
    ("lock",   (100, 100, 100, 255), (240, 240, 240, 255), "🔒"),
]

for name, bg, fg, text in BADGES:
    font = FONT_BIG if len(text) <= 1 else (FONT_MED if len(text) <= 2 else FONT_SMALL)
    badge(name, bg, fg, text, font)

# ---------------------------------------------------------------------------
# Folder icons
# ---------------------------------------------------------------------------
TEAL = (0, 229, 200, 255)
PURPLE = (199, 146, 234, 255)
AMBER = (255, 209, 112, 255)
RED = (255, 90, 90, 255)
GRAY = (120, 120, 130, 255)

folder_icon("folder",       TEAL)
folder_icon("folder_open",  TEAL, accent=(199, 146, 234, 255))
folder_icon("folder_dl",    (90, 180, 255, 255))   # downloads
folder_icon("folder_docs",  (180, 200, 255, 255))
folder_icon("folder_pics",  (255, 130, 200, 255))
folder_icon("folder_vid",   (255, 100, 100, 255))
folder_icon("folder_music", (220, 180, 255, 255))
folder_icon("folder_cfg",   GRAY)
folder_icon("folder_git",   (255, 140, 60, 255))
folder_icon("folder_proj",  (140, 220, 140, 255))
folder_icon("folder_node",  (76, 175, 80, 255))
folder_icon("folder_trash", (180, 80, 80, 255))
folder_icon("folder_cache", (120, 120, 80, 255))
folder_icon("folder_home",  PURPLE, accent=TEAL)

# ---------------------------------------------------------------------------
# Special files
# ---------------------------------------------------------------------------
# Distinct page-shaped icons for files that match by name not extension
file_icon("readme",      (199, 146, 234, 255), "i")
file_icon("license",     (255, 209, 112, 255), "©")
file_icon("makefile",    (90, 200, 90, 255),   "M")
file_icon("dockerfile",  (33, 150, 243, 255),  "🐳")
file_icon("cargo",       (222, 165, 132, 255), "C")
file_icon("gitignore",   (255, 140, 60, 255),  ".g")
file_icon("package",     (203, 56, 55, 255),   "{}")
file_icon("env",         (255, 209, 112, 255), "$=")

# ---------------------------------------------------------------------------
# Symlink
# ---------------------------------------------------------------------------
img = new_canvas()
d = ImageDraw.Draw(img)
rounded(d, (255, 209, 112, 255))
center_text(d, "↪", (10, 0, 16, 255), font=FONT_BIG)
img.save(OUT / "symlink.png")

# ---------------------------------------------------------------------------
# Generic fallback file
# ---------------------------------------------------------------------------
file_icon("file_generic", (160, 160, 180, 255), "·")

# ---------------------------------------------------------------------------
# Media (image/video/audio)
# ---------------------------------------------------------------------------
def media_icon(name, color, glyph):
    img = new_canvas()
    d = ImageDraw.Draw(img)
    rounded(d, color)
    center_text(d, glyph, (255, 255, 255, 255), font=FONT_BIG)
    img.save(OUT / f"{name}.png")

media_icon("image",   (255, 130, 200, 255), "▣")
media_icon("video",   (220, 80, 80, 255),   "▶")
media_icon("audio",   (180, 130, 220, 255), "♪")

print(f"wrote sprites → {OUT}")
print(f"total: {len(list(OUT.glob('*.png')))}")
