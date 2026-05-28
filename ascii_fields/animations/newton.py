import math

from ..core import Animation, clamp, render_field
from ._fractal import iteration_cap, newton_grid, zoom_scale


class NewtonAnimation(Animation):
  """Newton's-method fractal for z^3 = 1. The three roots slowly rotate, so the
  basin boundaries swirl, and the view drifts in and out."""

  thresholds = [
    (0.05, " "), (0.16, "."), (0.28, ":"), (0.40, "-"), (0.52, "="),
    (0.64, "+"), (0.76, "*"), (0.88, "#"), (1.01, "@"),
  ]

  def render(self, width, height, elapsed, phase, options):
    angle = elapsed * 0.25
    scale, _ = zoom_scale(elapsed * 0.4, base=1.7, depth=1.6, speed=0.5)
    max_iter = min(40, iteration_cap(width, height, lo=18, hi=40))
    grid = newton_grid(width, height, 0.0, 0.0, scale, angle, max_iter)
    contrast = options.contrast
    return render_field(width, height,
                        ([clamp(v * contrast) for v in row] for row in grid),
                        options, self)
