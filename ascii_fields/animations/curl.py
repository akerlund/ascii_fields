import random

from ..core import Animation, clamp, render_field
from ..noise import fbm


class CurlNoiseAnimation(Animation):
  """Many tiny particles drift through a divergence-free flow field built from
  the curl of fractal noise -- so they curl and braid through the screen but
  never converge into clumps. Trails fade behind each particle."""

  thresholds = [
    (0.06, " "), (0.20, "."), (0.34, ":"), (0.48, "-"), (0.62, "="),
    (0.74, "+"), (0.84, "*"), (0.93, "#"), (1.01, "@"),
  ]

  NOISE_SCALE = 0.10
  SPEED = 11.0
  TRAIL_DECAY = 0.90

  def __init__(self):
    self._w = 0
    self._h = 0
    self._particles = None
    self._trail = None
    self._last = 0.0
    self._rng = random.Random()

  def _seed(self, w, h):
    self._w, self._h = w, h
    n = max(80, (w * h) // 28)
    self._particles = [(self._rng.uniform(0, w), self._rng.uniform(0, h)) for _ in range(n)]
    self._trail = [0.0] * (w * h)

  def render(self, width, height, elapsed, phase, options):
    if self._particles is None or width != self._w or height != self._h or elapsed < self._last:
      self._seed(width, height)
    dt = clamp(elapsed - self._last, 0.0, 0.1) or 0.04
    self._last = elapsed

    trail = self._trail
    for i in range(len(trail)):
      trail[i] *= self.TRAIL_DECAY

    s = self.NOISE_SCALE * max(0.4, options.scale)
    speed = self.SPEED * max(0.4, options.scale)
    t_off = elapsed * 0.25
    new = []
    for (x, y) in self._particles:
      u = x * s + t_off
      v = y * s
      # curl: vx = d/dy N, vy = -d/dx N (divergence-free 2D flow)
      eps = 0.04
      nx_p = fbm(u + eps, v, octaves=3)
      nx_m = fbm(u - eps, v, octaves=3)
      ny_p = fbm(u, v + eps, octaves=3)
      ny_m = fbm(u, v - eps, octaves=3)
      vx = (ny_p - ny_m) / (2 * eps) * speed
      vy = -(nx_p - nx_m) / (2 * eps) * speed
      nx_pos = (x + vx * dt) % width
      ny_pos = (y + vy * dt) % height
      ci, ri = int(nx_pos), int(ny_pos)
      if 0 <= ci < width and 0 <= ri < height:
        idx = ri * width + ci
        if trail[idx] < 1.0:
          trail[idx] = min(1.0, trail[idx] + 0.5)
      new.append((nx_pos, ny_pos))
    self._particles = new

    contrast = options.contrast
    grid = []
    for r in range(height):
      base = r * width
      grid.append([clamp(trail[base + c] * contrast) for c in range(width)])
    return render_field(width, height, grid, options, self)
