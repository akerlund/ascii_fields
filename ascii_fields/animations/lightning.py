import math
import random

from ..core import Animation, clamp, render_field
from ..noise import fbm


STRIKE_PERIOD = 2.6
STRIKE_DURATION = 0.55


class LightningAnimation(Animation):
  """A night storm: dark roiling clouds, faint rain, and forked lightning bolts
  that flash and briefly light up the whole sky."""

  default_theme = "ice"

  thresholds = [
    (0.08, " "), (0.18, "."), (0.30, ":"), (0.42, "-"), (0.54, "="),
    (0.66, "+"), (0.78, "*"), (0.90, "#"), (1.01, "@"),
  ]

  def __init__(self):
    self._strike_idx = None
    self._centers = None       # per-row list of bolt x-centres
    self._w = self._h = 0

  def _build_bolt(self, seed, width, height):
    rng = random.Random(seed * 2654435761 & 0xFFFFFFFF)
    centers = [[] for _ in range(height)]
    x = rng.uniform(width * 0.25, width * 0.75)
    main = []
    for row in range(height):
      x += rng.uniform(-1.6, 1.6)
      if rng.random() < 0.12:
        x += rng.uniform(-4.0, 4.0)
      x = max(0.0, min(width - 1.0, x))
      main.append(x)
      centers[row].append(x)
    # a few diagonal branches peeling off the main channel
    for _ in range(rng.randint(2, 4) if height >= 6 else 0):
      start = rng.randint(1, height - 2)
      bx = main[start]
      direction = rng.choice((-1.0, 1.0))
      length = rng.randint(3, max(4, height // 3))
      for k in range(length):
        row = start + k
        if row >= height:
          break
        bx += direction * rng.uniform(0.8, 2.2) + rng.uniform(-0.6, 0.6)
        bx = max(0.0, min(width - 1.0, bx))
        centers[row].append(bx)
    return centers

  def _intensity(self, local):
    if local > STRIKE_DURATION:
      return 0.0
    flash = math.exp(-local / 0.10)
    flash += 0.6 * math.exp(-((local - 0.16) ** 2) / 0.0015)
    flash += 0.4 * math.exp(-((local - 0.30) ** 2) / 0.002)
    return min(1.0, flash)

  def render(self, width, height, elapsed, phase, options):
    idx = int(elapsed / STRIKE_PERIOD)
    local = elapsed - idx * STRIKE_PERIOD
    if idx != self._strike_idx or width != self._w or height != self._h:
      self._strike_idx = idx
      self._w, self._h = width, height
      self._centers = self._build_bolt(idx, width, height)
    intensity = self._intensity(local)
    sky_flash = 0.28 * intensity
    contrast = options.contrast

    grid = []
    for row in range(height):
      v = row / max(1, height - 1)
      centers = self._centers[row]
      line = []
      for col in range(width):
        u = col / max(1, width - 1)
        clouds = 0.10 + 0.20 * fbm(u * 4.0 + elapsed * 0.05, v * 2.0, octaves=4) * (1.0 - v * 0.5)
        rain = 0.05 * max(0.0, math.sin(40.0 * (v + u * 0.5) - elapsed * 9.0))
        level = clouds + rain + sky_flash * (0.6 + 0.4 * (1.0 - v))
        if intensity > 0.0 and centers:
          glow = 0.0
          for cxr in centers:
            dx = col - cxr
            glow = max(glow, math.exp(-(dx * dx) / 1.6))
          level += glow * (0.4 + 0.6 * intensity) * 1.4
        line.append(clamp(level * contrast))
      grid.append(line)
    return render_field(width, height, grid, options, self)
