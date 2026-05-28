import math
import random

from ..core import Animation, clamp, render_field


class ReactionDiffusionAnimation(Animation):
  """The Gray-Scott reaction-diffusion system. Two chemicals U and V react and
  diffuse on a torus; the V concentration paints organic spots, stripes and
  mazes that crawl and morph forever. Pure math, infinite variety."""

  thresholds = [
    (0.08, " "), (0.20, "."), (0.32, ":"), (0.44, "-"), (0.56, "="),
    (0.68, "+"), (0.80, "*"), (0.90, "#"), (1.01, "@"),
  ]

  DU = 0.16
  DV = 0.08
  F = 0.060
  K = 0.062
  STEPS_PER_FRAME = 6

  def __init__(self):
    self._w = 0
    self._h = 0
    self._U = None
    self._V = None
    self._last = 0.0

  def _seed(self, w, h):
    self._w, self._h = w, h
    n = w * h
    self._U = [1.0] * n
    self._V = [0.0] * n
    rng = random.Random(1)
    # a couple of seeded patches plus sparkles
    for cx, cy in ((w // 2, h // 2), (w // 3, h // 2), (2 * w // 3, h // 2)):
      for dy in range(-3, 4):
        for dx in range(-3, 4):
          x = (cx + dx) % w
          y = (cy + dy) % h
          if dx * dx + dy * dy <= 9:
            self._U[y * w + x] = 0.50
            self._V[y * w + x] = 0.25
    for _ in range(max(20, n // 80)):
      x = rng.randint(0, w - 1)
      y = rng.randint(0, h - 1)
      self._V[y * w + x] = 0.5

  def _step(self):
    w, h = self._w, self._h
    U, V = self._U, self._V
    DU, DV, F, K = self.DU, self.DV, self.F, self.K
    nU = U[:]
    nV = V[:]
    for y in range(h):
      ym = (y - 1) % h
      yp = (y + 1) % h
      base = y * w
      bm = ym * w
      bp = yp * w
      for x in range(w):
        xm = (x - 1) % w
        xp = (x + 1) % w
        u = U[base + x]
        v = V[base + x]
        lapU = U[base + xm] + U[base + xp] + U[bm + x] + U[bp + x] - 4 * u
        lapV = V[base + xm] + V[base + xp] + V[bm + x] + V[bp + x] - 4 * v
        uvv = u * v * v
        nU[base + x] = u + DU * lapU - uvv + F * (1.0 - u)
        nV[base + x] = v + DV * lapV + uvv - (F + K) * v
    self._U = nU
    self._V = nV

  def render(self, width, height, elapsed, phase, options):
    if self._U is None or width != self._w or height != self._h or elapsed < self._last:
      self._seed(width, height)
    self._last = elapsed
    steps = max(1, int(self.STEPS_PER_FRAME * max(0.4, options.scale)))
    # keep the simulation step budget bounded so big terminals stay smooth
    if width * height > 6000:
      steps = max(1, steps // 2)
    for _ in range(steps):
      self._step()
    contrast = options.contrast
    grid = []
    V = self._V
    for r in range(height):
      base = r * width
      grid.append([clamp(V[base + c] * 4.0 * contrast) for c in range(width)])
    return render_field(width, height, grid, options, self)
