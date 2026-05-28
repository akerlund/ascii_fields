import math

from ..core import Animation, clamp, render_field


class PlasmaAnimation(Animation):
  """The classic demoscene plasma: layered sine waves that ripple and breathe.
  Pairs beautifully with --theme plasma or --theme spectrum."""

  default_theme = "plasma"

  def render(self, width, height, elapsed, phase, options):
    t = elapsed
    freq = max(0.4, options.scale)
    contrast = options.contrast
    cx = 0.5 + 0.3 * math.sin(t * 0.6)
    cy = 0.5 + 0.3 * math.cos(t * 0.5)
    grid = []
    for row in range(height):
      v = row / max(1, height - 1)
      line = []
      for col in range(width):
        u = col / max(1, width - 1)
        x = u * 6.0 * freq
        y = v * 6.0 * freq
        value = math.sin(x + t)
        value += math.sin(y * 1.3 - t * 0.8)
        value += math.sin((x + y) * 0.7 + t * 0.5)
        d = math.hypot(u - cx, v - cy) * 10.0 * freq
        value += math.sin(d - t * 1.6)
        level = 0.5 + 0.5 * (value / 4.0)
        line.append(clamp(0.5 + (level - 0.5) * contrast))
      grid.append(line)
    return render_field(width, height, grid, options, self)
