import math

from ..core import Animation, clamp, render_field
from ..noise import fbm


class TunnelAnimation(Animation):
  """Flying through a winding tunnel. The vanishing point drifts, so the tunnel
  bends and curves as patterned walls rush past."""

  thresholds = [
    (0.10, " "), (0.20, "."), (0.32, ":"), (0.44, "-"), (0.56, "="),
    (0.68, "+"), (0.80, "*"), (0.90, "#"), (1.01, "@"),
  ]

  def render(self, width, height, elapsed, phase, options):
    t = elapsed
    aspect = width / max(1, height * 2.0)
    # the centre of the tunnel wanders, so the bore appears to wind
    cx = 0.5 + 0.22 * math.sin(t * 0.6)
    cy = 0.5 + 0.18 * math.cos(t * 0.8)
    rings = 7.0 * max(0.4, options.scale)
    contrast = options.contrast
    grid = []
    for row in range(height):
      v = row / max(1, height - 1)
      dy = v - cy
      line = []
      for col in range(width):
        u = col / max(1, width - 1)
        dx = (u - cx) * aspect
        r = math.hypot(dx, dy) + 1e-4
        angle = math.atan2(dy, dx)
        depth = 1.0 / r + t * 1.5            # texture rushes inward
        ring = 0.5 + 0.5 * math.sin(depth * rings)
        stripe = 0.5 + 0.5 * math.sin(angle * 8.0 + depth * 0.5)
        grime = fbm(angle * 2.0, depth * 0.4, octaves=3)
        wall = 0.35 * ring + 0.35 * stripe + 0.30 * grime
        lighting = clamp(r * 2.2)            # far centre is dark, near walls bright
        level = wall * lighting
        line.append(clamp(level * 1.4 * contrast))
      grid.append(line)
    return render_field(width, height, grid, options, self)
