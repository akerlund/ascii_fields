"""Render ASCII frames (with their ANSI colours) to an animated GIF via Pillow.

This parses the same escape-coded frame strings the live renderer produces, so a
GIF looks exactly like the terminal output -- grayscale or themed.
"""

import os
import re

_SGR = re.compile(r"\x1b\[([0-9;]*)m")

FONT_CANDIDATES = (
  "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf",
  "/usr/share/fonts/truetype/liberation/LiberationMono-Regular.ttf",
  "/usr/local/share/fonts/JetBrainsMonoNerd/JetBrainsMonoNLNerdFontMono-Regular.ttf",
)

DEFAULT_FG = (205, 205, 205)
DEFAULT_BG = (0, 0, 0)


def _xterm_rgb(n):
  if n < 16:
    base = (0, 95, 135, 175, 215, 255)
    table = [
      (0, 0, 0), (128, 0, 0), (0, 128, 0), (128, 128, 0),
      (0, 0, 128), (128, 0, 128), (0, 128, 128), (192, 192, 192),
      (128, 128, 128), (255, 0, 0), (0, 255, 0), (255, 255, 0),
      (0, 0, 255), (255, 0, 255), (0, 255, 255), (255, 255, 255),
    ]
    return table[n]
  if n < 232:
    n -= 16
    levels = (0, 95, 135, 175, 215, 255)
    return levels[n // 36], levels[(n // 6) % 6], levels[n % 6]
  gray = 8 + (n - 232) * 10
  return gray, gray, gray


def _apply_sgr(params, fg, bg):
  parts = [int(p) for p in params.split(";") if p != ""] or [0]
  i = 0
  while i < len(parts):
    code = parts[i]
    if code == 0:
      fg, bg = DEFAULT_FG, DEFAULT_BG
      i += 1
    elif code in (38, 48) and i + 1 < len(parts):
      mode = parts[i + 1]
      if mode == 5 and i + 2 < len(parts):
        rgb = _xterm_rgb(parts[i + 2])
        i += 3
      elif mode == 2 and i + 4 < len(parts):
        rgb = (parts[i + 2], parts[i + 3], parts[i + 4])
        i += 5
      else:
        i += 2
        continue
      if code == 38:
        fg = rgb
      else:
        bg = rgb
    else:
      i += 1
  return fg, bg


def parse_ansi(frame):
  """Return rows of (char, fg_rgb, bg_rgb) for a frame string."""
  rows = []
  for line in frame.split("\n"):
    cells = []
    fg, bg = DEFAULT_FG, DEFAULT_BG
    pos = 0
    for match in _SGR.finditer(line):
      for ch in line[pos:match.start()]:
        cells.append((ch, fg, bg))
      fg, bg = _apply_sgr(match.group(1), fg, bg)
      pos = match.end()
    for ch in line[pos:]:
      cells.append((ch, fg, bg))
    rows.append(cells)
  return rows


def _find_font():
  for path in FONT_CANDIDATES:
    if os.path.exists(path):
      return path
  return None


def write_gif(frames, path, fps, font_size=16):
  try:
    from PIL import Image, ImageDraw, ImageFont
  except ImportError as exc:  # pragma: no cover
    raise SystemExit("GIF export needs Pillow. Install it with: pip install pillow") from exc

  font_path = _find_font()
  font = ImageFont.truetype(font_path, font_size) if font_path else ImageFont.load_default()
  cell_w = max(1, int(round(font.getlength("M")))) if font_path else 8
  ascent, descent = font.getmetrics()
  cell_h = ascent + descent

  images = []
  for frame in frames:
    rows = parse_ansi(frame)
    height = len(rows)
    width = max((len(r) for r in rows), default=1)
    img = Image.new("RGB", (cell_w * width, cell_h * height), DEFAULT_BG)
    draw = ImageDraw.Draw(img)
    for y, row in enumerate(rows):
      py = y * cell_h
      for x, (ch, fg, bg) in enumerate(row):
        px = x * cell_w
        if bg != DEFAULT_BG:
          draw.rectangle([px, py, px + cell_w, py + cell_h], fill=bg)
        if ch != " ":
          draw.text((px, py), ch, font=font, fill=fg)
    images.append(img)

  if not images:
    raise SystemExit("No frames to write.")
  duration = max(20, int(1000 / max(1.0, fps)))
  images[0].save(path, save_all=True, append_images=images[1:],
                 duration=duration, loop=0, optimize=True)
  return len(images), images[0].size
