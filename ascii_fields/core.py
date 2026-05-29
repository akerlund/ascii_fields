"""Shared primitives: render options, math helpers, and the field renderer.

Every scene builds a 2D grid of brightness levels in [0, 1] and hands it to
:func:`render_field` (or :func:`render_block_field`). That keeps the
performance-critical ANSI batching and palette logic in one place, so colour
themes and optimisations only have to be written once.
"""

from dataclasses import dataclass

from . import themes


RESET = "\x1b[0m"
HIDE_CURSOR = "\x1b[?25l"
SHOW_CURSOR = "\x1b[?25h"
HOME = "\x1b[H"
CLEAR = "\x1b[2J"
ALT_SCREEN_ON = "\x1b[?1049h"
ALT_SCREEN_OFF = "\x1b[?1049l"
BLACK_BG = "\x1b[48;5;232m"


@dataclass
class RenderOptions:
  """Render-time knobs. Not frozen -- the HUD mutates these in place at runtime
  so theme, scale, contrast, and brightness can be retuned live."""
  scale: float = 1.0
  contrast: float = 1.05
  brightness: float = 1.0
  charset: str = "clean"
  scroll: bool = False
  ascii_mode: bool = True
  blocks: bool = False
  theme: str = "auto"


def clamp(value, low=0.0, high=1.0):
  return low if value < low else high if value > high else value


def smoothstep(edge0, edge1, value):
  value = clamp((value - edge0) / (edge1 - edge0))
  return value * value * (3.0 - 2.0 * value)


def shade(value, contrast):
  return clamp(0.5 + value * 0.42 * contrast)


def star_noise(ix, iy):
  value = (ix * 374761393 + iy * 668265263) & 0xFFFFFFFF
  value = (value ^ (value >> 13)) * 1274126177
  value = value ^ (value >> 16)
  return (value & 0xFFFF) / 65535.0


def density_char(level, thresholds):
  for limit, char in thresholds:
    if level < limit:
      return char
  return thresholds[-1][1]


def gray_fg(level, lo=234, hi=255, brightness=1.0):
  value = clamp(level * brightness)
  return lo + round(value * (hi - lo))


class Animation:
  """Base class. Subclasses implement :meth:`render` returning a frame string.

  ``thresholds`` is the level -> character ramp. ``gray_lo``/``gray_hi`` bound
  the 256-colour grayscale ramp (234..255 keeps a dark floor with bright
  highlights). ``default_theme`` is used when the user does not pass --theme.
  """

  thresholds = [
    (0.12, " "), (0.20, "."), (0.30, ":"), (0.41, "-"), (0.53, "="),
    (0.66, "+"), (0.79, "*"), (0.91, "#"), (1.01, "%"),
  ]
  gray_lo = 234
  gray_hi = 255
  default_theme = "mono"

  def render(self, width, height, elapsed, phase, options):
    raise NotImplementedError


def resolve_theme(options, default_theme):
  """Map the requested theme to a concrete palette name.

  ``auto`` keeps the classic grayscale look for every scene; ``scene`` uses each
  animation's hand-picked colour theme; anything else forces that theme.
  """
  name = options.theme
  if name == "auto":
    return "mono"
  if name == "scene":
    return default_theme
  return name


def render_field(width, height, grid, options, animation=None):
  """Turn a grid of [0, 1] levels into a dark-background ASCII frame string.

  ``grid`` is an iterable of ``height`` rows, each an iterable of ``width``
  floats. Run-length batches colour escapes so only changes are emitted.
  """
  animation = animation or Animation
  thresholds = getattr(animation, "thresholds", Animation.thresholds)
  lo = getattr(animation, "gray_lo", 234)
  hi = getattr(animation, "gray_hi", 255)
  default_theme = getattr(animation, "default_theme", "mono")
  theme = resolve_theme(options, default_theme)
  bright = options.brightness

  rows_out = []
  if theme == "mono":
    for row in grid:
      line = [BLACK_BG]
      last = None
      for level in row:
        color = gray_fg(level, lo, hi, bright)
        char = density_char(level, thresholds)
        if color != last:
          line.append(f"\x1b[38;5;{color}m")
          last = color
        line.append(char)
      line.append(RESET)
      rows_out.append("".join(line))
  else:
    palette = themes.palette(theme)
    for row in grid:
      line = [BLACK_BG]
      last = None
      for level in row:
        char = density_char(level, thresholds)
        rgb = palette(clamp(level * bright))
        if rgb != last:
          line.append(f"\x1b[38;2;{rgb[0]};{rgb[1]};{rgb[2]}m")
          last = rgb
        line.append(char)
      line.append(RESET)
      rows_out.append("".join(line))
  return "\n".join(rows_out)


def render_glyph_field(width, height, grid, glyphs, options, animation=None):
  """Like :func:`render_field` but each cell's character is supplied explicitly
  via ``glyphs`` (a matching grid of single characters). Used by scenes that
  draw real symbols rather than a density ramp (e.g. the Matrix rain)."""
  animation = animation or Animation
  lo = getattr(animation, "gray_lo", 234)
  hi = getattr(animation, "gray_hi", 255)
  default_theme = getattr(animation, "default_theme", "mono")
  theme = resolve_theme(options, default_theme)
  bright = options.brightness
  palette = None if theme == "mono" else themes.palette(theme)

  rows_out = []
  for row, glyph_row in zip(grid, glyphs):
    line = [BLACK_BG]
    last = None
    for level, glyph in zip(row, glyph_row):
      if level <= 0.001:
        line.append(" ")
        continue
      if palette is None:
        color = gray_fg(level, lo, hi, bright)
        if color != last:
          line.append(f"\x1b[38;5;{color}m")
          last = color
      else:
        rgb = palette(clamp(level * bright))
        if rgb != last:
          line.append(f"\x1b[38;2;{rgb[0]};{rgb[1]};{rgb[2]}m")
          last = rgb
      line.append(glyph)
    line.append(RESET)
    rows_out.append("".join(line))
  return "\n".join(rows_out)


def render_block_field(width, height, grid, options):
  """Solid-colour block frame (no characters), used by the wave plane."""
  theme = resolve_theme(options, "mono")
  bright = options.brightness
  rows_out = []
  if theme == "mono":
    for row in grid:
      line = []
      last = None
      for level in row:
        gray = 232 + round(clamp(level * bright) * 23)
        if gray != last:
          line.append(f"\x1b[48;5;{gray}m")
          last = gray
        line.append(" ")
      line.append(RESET)
      rows_out.append("".join(line))
  else:
    palette = themes.palette(theme)
    for row in grid:
      line = []
      last = None
      for level in row:
        rgb = palette(clamp(level * bright))
        if rgb != last:
          line.append(f"\x1b[48;2;{rgb[0]};{rgb[1]};{rgb[2]}m")
          last = rgb
        line.append(" ")
      line.append(RESET)
      rows_out.append("".join(line))
  return "\n".join(rows_out)
