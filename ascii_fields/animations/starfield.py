import math
import random

from ..core import Animation, clamp, render_field


class StarfieldAnimation(Animation):
  """Flying forward through a 3D starfield. Stars stream out from the vanishing
  point, brightening and leaving short motion trails as they rush past."""

  thresholds = [
    (0.05, " "), (0.20, "."), (0.40, ":"), (0.58, "+"), (0.74, "*"),
    (0.86, "o"), (0.94, "#"), (1.01, "@"),
  ]

  STAR_DENSITY = 0.05       # stars per cell

  def __init__(self):
    self._stars = None
    self._w = self._h = 0
    self._last_elapsed = 0.0
    self._rng = random.Random()

  def _seed(self, w, h):
    self._w, self._h = w, h
    count = max(40, int(w * h * self.STAR_DENSITY))
    self._stars = [self._new_star() for _ in range(count)]

  def _new_star(self, z=None):
    return [
      self._rng.uniform(-1.0, 1.0),                 # x
      self._rng.uniform(-1.0, 1.0),                 # y
      self._rng.uniform(0.1, 1.0) if z is None else z,  # z (depth)
    ]

  def render(self, width, height, elapsed, phase, options):
    if self._stars is None or width != self._w or height != self._h or elapsed < self._last_elapsed:
      self._seed(width, height)
    dt = clamp(elapsed - self._last_elapsed, 0.0, 0.1)
    self._last_elapsed = elapsed
    speed = 0.55 * max(0.4, options.scale)

    grid = [[0.0] * width for _ in range(height)]
    aspect = height * 2.0 / max(1, width)
    cx, cy = width * 0.5, height * 0.5
    fov = 0.9
    for star in self._stars:
      star[2] -= speed * dt
      if star[2] <= 0.02:
        star[0] = self._rng.uniform(-1.0, 1.0)
        star[1] = self._rng.uniform(-1.0, 1.0)
        star[2] = 1.0
      z = star[2]
      sx = cx + (star[0] / z) * fov * cx
      sy = cy + (star[1] / z) * fov * cy * aspect
      bright = clamp((1.0 - z) ** 1.5 + 0.15)
      # short trail back toward the vanishing point
      tz = min(1.0, z + speed * 0.06)
      tx = cx + (star[0] / tz) * fov * cx
      ty = cy + (star[1] / tz) * fov * cy * aspect
      steps = max(1, int(math.hypot(sx - tx, sy - ty)))
      for s in range(steps + 1):
        f = s / max(1, steps)
        px = int(round(tx + (sx - tx) * f))
        py = int(round(ty + (sy - ty) * f))
        if 0 <= px < width and 0 <= py < height:
          val = bright * (0.4 + 0.6 * f)
          if val > grid[py][px]:
            grid[py][px] = val
    return render_field(width, height, grid, options, self)
