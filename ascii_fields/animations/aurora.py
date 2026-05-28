import math

from ..core import Animation, clamp, render_field
from ..noise import fbm


class AuroraAnimation(Animation):
  """Northern lights: shimmering vertical curtains that ripple along the sky,
  with faint rays streaking up and a dim star field behind. Try --theme aurora."""

  default_theme = "aurora"

  thresholds = [
    (0.08, " "), (0.18, "."), (0.30, ":"), (0.42, "-"), (0.55, "="),
    (0.68, "+"), (0.80, "*"), (0.90, "#"), (1.01, "@"),
  ]

  # (vertical centre, thickness, horizontal frequency, drift speed, weight)
  CURTAINS = (
    (0.42, 0.14, 2.3, 0.55, 1.00),
    (0.52, 0.10, 3.7, -0.40, 0.75),
    (0.34, 0.08, 5.1, 0.70, 0.55),
  )

  def render(self, width, height, elapsed, phase, options):
    contrast = options.contrast
    grid = []
    for row in range(height):
      v = row / max(1, height - 1)
      line = []
      for col in range(width):
        u = col / max(1, width - 1)
        value = 0.04
        for centre, thick, freq, speed, weight in self.CURTAINS:
          cy = centre + 0.10 * math.sin(freq * u * math.pi + elapsed * speed)
          cy += 0.05 * fbm(u * 3.0 + elapsed * 0.1, centre * 4.0, octaves=3)
          band = math.exp(-((v - cy) ** 2) / (thick * thick))
          rays = 0.55 + 0.45 * math.sin(u * 60.0 + 8.0 * fbm(u * 6.0, elapsed * 0.3))
          fade = clamp(1.0 - v * 0.4)
          value += weight * band * rays * fade
        line.append(clamp(value * contrast))
      grid.append(line)
    return render_field(width, height, grid, options, self)
