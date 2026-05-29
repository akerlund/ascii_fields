import math

from ..core import Animation, clamp, render_field
from ._fractal import iteration_cap, mandelbrot_grid


CYCLE_SECONDS = 28.0
DEPTH = 11.0       # log-zoom depth over one cycle (~ exp(11) ~= 60k magnification)


class MandelbrotAnimation(Animation):
  """Continuous zoom into the Mandelbrot set aimed at a detailed point in the
  seahorse valley. The zoom never reverses -- it keeps diving deeper until the
  cycle wraps back to the starting frame (the same first figure) and dives
  again."""

  thresholds = [
    (0.04, " "), (0.13, "."), (0.25, ":"), (0.38, "-"), (0.52, "="),
    (0.66, "+"), (0.78, "*"), (0.90, "#"), (1.01, "@"),
  ]

  CENTER = (-0.743643887037151, 0.131825904205330)
  BASE_SCALE = 1.6
  MIN_VISIBLE = 0.26

  def render(self, width, height, elapsed, phase, options):
    # sawtooth: zoom in continuously, wrap back to the starting frame each cycle
    frac = (elapsed / CYCLE_SECONDS) % 1.0
    scale = self.BASE_SCALE * math.exp(-DEPTH * frac)
    max_iter = iteration_cap(width, height)
    cx, cy = self.CENTER
    grid = mandelbrot_grid(width, height, cx, cy, scale, max_iter)
    grid = self._keep_visible(grid, width, height)
    contrast = options.contrast
    return render_field(width, height,
                        ([clamp(v * contrast) for v in row] for row in grid),
                        options, self)

  def _keep_visible(self, grid, width, height):
    values = [value for row in grid for value in row]
    if not values:
      return grid
    visible = sum(value >= self.thresholds[0][0] for value in values) / max(1, width * height)
    high = max(values)
    if visible >= self.MIN_VISIBLE or high <= 0.0:
      return grid
    # Deep valley frames can have real detail that lands below the first glyph
    # threshold. Lift those escape values into the ramp instead of flashing dark.
    floor = self.thresholds[0][0]
    return [
      [0.0 if value <= 0.0 else floor + (value / high) ** 0.55 * (1.0 - floor)
       for value in row]
      for row in grid
    ]
