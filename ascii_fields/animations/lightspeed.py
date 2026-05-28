import math
import random

from ..core import Animation, clamp, render_field


CYCLE = 9.0


def _smoothstep(a, b, x):
  t = clamp((x - a) / (b - a))
  return t * t * (3.0 - 2.0 * t)


class LightspeedAnimation(Animation):
  """Punch it. Stars sit calm, then stretch into streaks as the ship winds up,
  a white flash marks the jump, and the sky becomes the streaking starlines of
  hyperspace before dropping back to cruise. Try --theme ice."""

  default_theme = "ice"

  thresholds = [
    (0.05, " "), (0.20, "."), (0.40, ":"), (0.58, "-"), (0.72, "+"),
    (0.84, "*"), (0.93, "#"), (1.01, "@"),
  ]

  def __init__(self):
    self._stars = None
    self._rng = random.Random()

  def _seed(self, n):
    self._stars = [(self._rng.uniform(0.0, math.tau), self._rng.uniform(0.0, 1.0))
                   for _ in range(n)]

  def render(self, width, height, elapsed, phase, options):
    n = max(60, (width * height) // 18)
    if self._stars is None or len(self._stars) != n:
      self._seed(n)

    p = (elapsed % CYCLE) / CYCLE
    streak = clamp(_smoothstep(0.40, 0.60, p) - _smoothstep(0.90, 1.0, p))
    flash = math.exp(-((p - 0.605) ** 2) / 0.0006)
    shimmer = 0.85 + 0.15 * math.sin(elapsed * 30.0)

    cx, cy = width * 0.5, height * 0.5
    rmax = math.hypot(cx, cy)
    flow = elapsed * (0.04 + streak * 1.4)
    length = 0.8 + streak * rmax * 1.05 * shimmer
    head_bright = 0.45 + 0.5 * streak

    grid = [[0.0] * width for _ in range(height)]
    for ang, r0 in self._stars:
      ca, sa = math.cos(ang), math.sin(ang) * 0.5
      r = ((r0 + flow) % 1.0) * rmax
      x0, y0 = cx + ca * r, cy + sa * r
      x1, y1 = cx + ca * (r + length), cy + sa * (r + length)
      steps = max(1, int(math.hypot(x1 - x0, y1 - y0)))
      for s in range(steps + 1):
        f = s / steps
        px = int(round(x0 + (x1 - x0) * f))
        py = int(round(y0 + (y1 - y0) * f))
        if 0 <= px < width and 0 <= py < height:
          val = head_bright * (0.35 + 0.65 * f)
          if val > grid[py][px]:
            grid[py][px] = val

    if flash > 0.01:
      grid = [[clamp(v + flash) for v in row] for row in grid]
    return render_field(width, height, grid, options, self)
