import random

from ..core import Animation, clamp, render_field


class DLAAnimation(Animation):
  """Diffusion-limited aggregation. Random walkers wander from the edge until
  they bump into the growing cluster and freeze. The result is a delicate
  branched fractal that grows like frost on a window. Newer branches are
  brighter than the original seed."""

  thresholds = [
    (0.05, " "), (0.20, "."), (0.34, ":"), (0.48, "-"), (0.62, "="),
    (0.74, "+"), (0.85, "*"), (0.93, "#"), (1.01, "@"),
  ]

  MAX_WALK = 1500

  def __init__(self):
    self._w = 0
    self._h = 0
    self._cluster = None
    self._age = None
    self._gen = 0
    self._last = 0.0
    self._rng = random.Random()

  def _seed(self, w, h):
    self._w, self._h = w, h
    self._cluster = [False] * (w * h)
    self._age = [0] * (w * h)
    cx, cy = w // 2, h // 2
    self._cluster[cy * w + cx] = True
    self._age[cy * w + cx] = 1
    self._gen = 1

  def _walk_one(self):
    w, h = self._w, self._h
    rng = self._rng
    cluster = self._cluster
    # spawn at a random empty cell
    for _ in range(20):
      x = rng.randint(0, w - 1)
      y = rng.randint(0, h - 1)
      if not cluster[y * w + x]:
        break
    else:
      return
    for _ in range(self.MAX_WALK):
      base = y * w
      # check the four neighbours (wrapped) -- stick on any contact
      for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
        nx = (x + dx) % w
        ny = (y + dy) % h
        if cluster[ny * w + nx]:
          cluster[base + x] = True
          self._gen += 1
          self._age[base + x] = self._gen
          return
      dx, dy = rng.choice(((1, 0), (-1, 0), (0, 1), (0, -1)))
      x = (x + dx) % w
      y = (y + dy) % h

  def render(self, width, height, elapsed, phase, options):
    if self._cluster is None or width != self._w or height != self._h or elapsed < self._last:
      self._seed(width, height)
    self._last = elapsed
    walkers = max(20, (width * height) // 80) * max(1, int(options.scale))
    # cap total density: stop growing once ~25% of cells are cluster
    if sum(self._cluster) < (width * height) // 4:
      for _ in range(walkers):
        self._walk_one()

    max_age = max(1, self._gen)
    contrast = options.contrast
    grid = []
    age = self._age
    for r in range(height):
      base = r * width
      row = []
      for c in range(width):
        a = age[base + c]
        if a:
          row.append(clamp((0.22 + 0.78 * (a / max_age)) * contrast))
        else:
          row.append(0.0)
      grid.append(row)
    return render_field(width, height, grid, options, self)
