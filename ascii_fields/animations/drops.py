import math
import random

from ..core import Animation, clamp, render_field


class _Drop:
  __slots__ = ("x", "y", "t0", "strength")

  def __init__(self, x, y, t0, strength):
    self.x = x
    self.y = y
    self.t0 = t0
    self.strength = strength


class DropsAnimation(Animation):
  """Water drops falling into a pond. Each impact sends out an expanding ring;
  overlapping rings interfere the way real ripples do."""

  thresholds = [
    (0.30, "."), (0.40, ":"), (0.47, "-"), (0.53, "="), (0.60, "+"),
    (0.70, "*"), (0.82, "#"), (1.01, "%"),
  ]

  WAVE_SPEED = 0.55       # rings per second outward (normalised units)
  SPAWN_DT = 0.55

  def __init__(self):
    self._drops = []
    self._next_spawn = 0.0
    self._last_elapsed = 0.0
    self._rng = random.Random()

  def _reset(self):
    self._drops = []
    self._next_spawn = 0.0

  def render(self, width, height, elapsed, phase, options):
    if elapsed < self._last_elapsed:
      self._reset()
    self._last_elapsed = elapsed

    aspect = width / max(1, height * 2.0)
    while elapsed >= self._next_spawn:
      self._drops.append(_Drop(
        self._rng.uniform(0.08, 0.92),
        self._rng.uniform(0.08, 0.92),
        self._next_spawn,
        self._rng.uniform(0.7, 1.0),
      ))
      self._next_spawn += self.SPAWN_DT * self._rng.uniform(0.6, 1.5)
    self._drops = [d for d in self._drops if elapsed - d.t0 < 7.0][-16:]

    contrast = options.contrast
    grid = []
    for row in range(height):
      v = row / max(1, height - 1)
      line = []
      for col in range(width):
        u = col / max(1, width - 1)
        h = 0.0
        for d in self._drops:
          age = elapsed - d.t0
          dx = (u - d.x) * aspect
          dy = v - d.y
          r = math.hypot(dx, dy)
          front = self.WAVE_SPEED * age
          ring = r - front
          envelope = math.exp(-(ring * ring) / 0.010)
          decay = math.exp(-age * 0.5) / (1.0 + 6.0 * r)
          h += d.strength * envelope * decay * math.sin(38.0 * ring)
        line.append(clamp(0.42 + h * 2.4 * contrast))
      grid.append(line)
    return render_field(width, height, grid, options, self)
