import math
import random

from ..core import Animation, clamp, render_field
from ..noise import fbm


class _Pair:
  """A virtual particle-antiparticle pair: born, drift apart, recombine, flash."""
  __slots__ = ("x", "y", "angle", "t0", "life", "reach")

  def __init__(self, x, y, angle, t0, life, reach):
    self.x = x
    self.y = y
    self.angle = angle
    self.t0 = t0
    self.life = life
    self.reach = reach


class QuantumFieldAnimation(Animation):
  """Vacuum fluctuations: a rippling energy field on which particle-antiparticle
  pairs continually bubble into existence, separate, then annihilate."""

  thresholds = [
    (0.10, " "), (0.20, "."), (0.32, ":"), (0.44, "-"), (0.56, "="),
    (0.68, "+"), (0.80, "*"), (0.90, "#"), (1.01, "@"),
  ]

  SPAWN_DT = 0.22

  def __init__(self):
    self._pairs = []
    self._next = 0.0
    self._last_elapsed = 0.0
    self._rng = random.Random()

  def render(self, width, height, elapsed, phase, options):
    if elapsed < self._last_elapsed:
      self._pairs = []
      self._next = 0.0
    self._last_elapsed = elapsed

    while elapsed >= self._next:
      self._pairs.append(_Pair(
        self._rng.uniform(0.05, 0.95),
        self._rng.uniform(0.05, 0.95),
        self._rng.uniform(0.0, math.tau),
        self._next,
        self._rng.uniform(0.5, 1.1),
        self._rng.uniform(0.04, 0.10),
      ))
      self._next += self.SPAWN_DT * self._rng.uniform(0.4, 1.6)
    self._pairs = [p for p in self._pairs if elapsed - p.t0 < p.life]

    aspect = width / max(1, height * 2.0)
    contrast = options.contrast
    # precompute pair endpoints for this frame
    active = []
    for p in self._pairs:
      age = (elapsed - p.t0) / p.life          # 0..1
      sep = math.sin(age * math.pi) * p.reach  # out then back in
      ca, sa = math.cos(p.angle), math.sin(p.angle)
      intensity = 0.5 + 0.5 * math.sin(age * math.pi)
      if age > 0.85:
        intensity += (age - 0.85) / 0.15       # annihilation flash
      active.append((p.x + ca * sep, p.y + sa * sep, p.x - ca * sep, p.y - sa * sep, intensity))

    grid = []
    for row in range(height):
      v = row / max(1, height - 1)
      line = []
      for col in range(width):
        u = col / max(1, width - 1)
        field = 0.14 + 0.30 * fbm(u * 6.0 + elapsed * 0.4, v * 6.0 - elapsed * 0.3, octaves=4)
        spark = 0.0
        for (ax, ay, bx, by, inten) in active:
          dxa = (u - ax) * aspect
          dya = v - ay
          dxb = (u - bx) * aspect
          dyb = v - by
          spark += inten * math.exp(-(dxa * dxa + dya * dya) / 0.0008)
          spark += inten * math.exp(-(dxb * dxb + dyb * dyb) / 0.0008)
        line.append(clamp((field + spark) * contrast))
      grid.append(line)
    return render_field(width, height, grid, options, self)
