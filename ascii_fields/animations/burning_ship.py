from ..core import Animation, clamp, render_field
from ._fractal import iteration_cap, mandelbrot_grid, zoom_scale


class BurningShipAnimation(Animation):
  """Endless zoom into the Burning Ship fractal -- the jagged, flame-like
  cousin of the Mandelbrot set, formed by taking absolute values each step."""

  thresholds = [
    (0.04, " "), (0.13, "."), (0.25, ":"), (0.38, "-"), (0.52, "="),
    (0.66, "+"), (0.78, "*"), (0.90, "#"), (1.01, "@"),
  ]

  CENTER = (-1.7549, -0.0260)

  def render(self, width, height, elapsed, phase, options):
    scale, _ = zoom_scale(elapsed * 0.6, base=0.9, depth=9.0, speed=0.5)
    max_iter = iteration_cap(width, height)
    cx, cy = self.CENTER
    grid = mandelbrot_grid(width, height, cx, cy, scale, max_iter, ship=True)
    contrast = options.contrast
    return render_field(width, height,
                        ([clamp(v * contrast) for v in row] for row in grid),
                        options, self)
