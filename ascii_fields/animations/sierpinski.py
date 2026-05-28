import math

from ..core import Animation, clamp, render_field


MAX_BITS = 15


def _membership(x, y):
  """Depth at which (x, y) in [0,1)^2 falls into a removed hole of the
  right-triangle Sierpinski gasket. Gasket points survive all levels."""
  for i in range(MAX_BITS):
    x *= 2.0
    y *= 2.0
    bx = x >= 1.0
    by = y >= 1.0
    if bx:
      x -= 1.0
    if by:
      y -= 1.0
    if bx and by:
      return i
  return MAX_BITS


class SierpinskiAnimation(Animation):
  """A Sierpinski triangle that zooms in forever. The gasket is self-similar
  under halving toward the corner, so each octave loops seamlessly into the
  next -- an endless descent into smaller and smaller triangles."""

  thresholds = [
    (0.06, " "), (0.22, "."), (0.36, ":"), (0.50, "-"), (0.62, "="),
    (0.74, "+"), (0.84, "*"), (0.93, "#"), (1.01, "@"),
  ]

  SPEED = 0.32   # octaves per second

  def render(self, width, height, elapsed, phase, options):
    frac = (elapsed * self.SPEED) % 1.0
    window = math.pow(2.0, -frac)        # 1.0 -> 0.5, then wraps seamlessly
    contrast = options.contrast
    grid = []
    for row in range(height):
      y = (1.0 - row / max(1, height - 1)) * window
      line = []
      for col in range(width):
        x = (col / max(1, width - 1)) * window
        depth = _membership(x, y)
        level = depth / MAX_BITS
        line.append(clamp(level * contrast))
      grid.append(line)
    return render_field(width, height, grid, options, self)
