import math

from ..core import Animation, clamp, render_field
from ..noise import fbm


class WhirlpoolAnimation(Animation):
  """A maelstrom: water spiralling inward and down a dark central throat, with
  logarithmic spiral arms, foam streaks and a churning surface."""

  thresholds = [
    (0.10, " "), (0.20, "."), (0.32, ":"), (0.44, "-"), (0.56, "="),
    (0.68, "+"), (0.80, "*"), (0.90, "#"), (1.01, "@"),
  ]

  def render(self, width, height, elapsed, phase, options):
    t = elapsed
    arms = 2
    windings = 5.0 * max(0.5, options.scale)
    contrast = options.contrast
    radius = max(1.0, min(width, height * 2) * 0.5)
    grid = []
    for row in range(height):
      py = ((row - (height - 1) * 0.5) * 2.0) / radius
      line = []
      for col in range(width):
        px = (col - (width - 1) * 0.5) / radius
        r = math.hypot(px, py) + 1e-4
        theta = math.atan2(py, px)
        # faster rotation toward the centre (differential swirl)
        swirl = theta + windings * math.log(r + 0.05) + t * (0.6 + 1.4 / (1.0 + 8.0 * r))
        spiral = 0.5 + 0.5 * math.sin(arms * swirl)
        foam = fbm(math.cos(swirl) * 3.0 + t * 0.2, math.sin(swirl) * 3.0, octaves=4)
        surface = 0.45 * spiral + 0.45 * foam
        throat = 1.0 - math.exp(-(r * r) / 0.02)       # dark hole in the middle
        rim = math.exp(-((r - 0.16) ** 2) / 0.01) * 0.6
        edge = clamp(1.0 - (r - 1.0) * 2.5)            # fade past the rim
        level = (surface * throat + rim) * edge
        line.append(clamp(level * 1.5 * contrast))
      grid.append(line)
    return render_field(width, height, grid, options, self)
