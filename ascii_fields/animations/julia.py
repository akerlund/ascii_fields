import math

from ..core import Animation, clamp, render_field
from ._fractal import iteration_cap, julia_grid, zoom_scale


class JuliaAnimation(Animation):
  """A Julia set whose constant c slowly orbits a circle, continuously morphing
  the shape, while the view gently breathes in and out."""

  thresholds = [
    (0.04, " "), (0.13, "."), (0.25, ":"), (0.38, "-"), (0.52, "="),
    (0.66, "+"), (0.78, "*"), (0.90, "#"), (1.01, "@"),
  ]

  def render(self, width, height, elapsed, phase, options):
    a = elapsed * 0.13
    radius = 0.7885
    cr = radius * math.cos(a)
    ci = radius * math.sin(a)
    scale, _ = zoom_scale(elapsed * 0.5, base=1.6, depth=2.2, speed=0.6)
    max_iter = iteration_cap(width, height)
    grid = julia_grid(width, height, 0.0, 0.0, scale, cr, ci, max_iter)
    contrast = options.contrast
    return render_field(width, height,
                        ([clamp(v * contrast) for v in row] for row in grid),
                        options, self)
