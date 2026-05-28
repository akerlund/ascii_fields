from ..core import Animation, clamp, render_field
from ._fractal import iteration_cap, mandelbrot_grid, zoom_scale


class MandelbrotAnimation(Animation):
  """Endless zoom in and back out of the Mandelbrot set, aimed at a detailed
  point in the seahorse valley."""

  thresholds = [
    (0.04, " "), (0.13, "."), (0.25, ":"), (0.38, "-"), (0.52, "="),
    (0.66, "+"), (0.78, "*"), (0.90, "#"), (1.01, "@"),
  ]

  CENTER = (-0.743643887037151, 0.131825904205330)

  def render(self, width, height, elapsed, phase, options):
    scale, _ = zoom_scale(elapsed * 0.6, base=1.6, depth=12.0, speed=0.5)
    max_iter = iteration_cap(width, height)
    cx, cy = self.CENTER
    grid = mandelbrot_grid(width, height, cx, cy, scale, max_iter)
    contrast = options.contrast
    return render_field(width, height,
                        ([clamp(v * contrast) for v in row] for row in grid),
                        options, self)
