import math
import random

from ..core import Animation, clamp, render_glyph_field


GLYPHS = "abcdefghijklmnopqrstuvwxyz0123456789@#$%&*+=<>|/\\:;.-"


class RainAnimation(Animation):
  """Matrix-style digital rain: columns of glyphs streaming downward with a
  bright leading head and a fading green-grey trail."""

  default_theme = "aurora"   # looks great in green truecolor; mono by default

  def __init__(self):
    self._w = 0
    self._h = 0
    self._heads = None
    self._speeds = None
    self._lengths = None
    self._rng = random.Random()

  def _seed(self, w, h):
    self._w, self._h = w, h
    self._offsets = [self._rng.uniform(0, h) for _ in range(w)]
    self._speeds = [self._rng.uniform(6.0, 18.0) for _ in range(w)]
    self._lengths = [self._rng.randint(5, max(6, h // 2)) for _ in range(w)]

  def render(self, width, height, elapsed, phase, options):
    if self._heads is None or width != self._w or height != self._h:
      self._heads = True
      self._seed(width, height)
    speed_mul = max(0.4, options.scale)
    heads = [(self._offsets[c] + elapsed * self._speeds[c] * speed_mul)
             % (height + self._lengths[c]) for c in range(width)]
    flick = int(elapsed * 12)
    grid = []
    glyphs = []
    for row in range(height):
      level_row = []
      glyph_row = []
      for col in range(width):
        head = heads[col]
        d = head - row
        length = self._lengths[col]
        if 0.0 <= d < length:
          if d < 1.0:
            level = 1.0
          else:
            level = clamp((1.0 - d / length) * 0.85)
          gi = (col * 31 + row * 17 + (flick if d < 2 else flick // 6)) % len(GLYPHS)
          glyph_row.append(GLYPHS[gi])
          level_row.append(level)
        else:
          glyph_row.append(" ")
          level_row.append(0.0)
      grid.append(level_row)
      glyphs.append(glyph_row)
    return render_glyph_field(width, height, grid, glyphs, options, self)
