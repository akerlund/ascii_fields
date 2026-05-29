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
    (0.42, 0.15, 2.1, 0.22, 1.00),
    (0.53, 0.11, 3.2, -0.18, 0.72),
    (0.33, 0.09, 4.4, 0.28, 0.56),
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
          drift = elapsed * speed
          cy = centre + 0.10 * math.sin(freq * u * math.pi + drift)
          cy += 0.045 * math.sin(u * 8.0 + elapsed * 0.33 + centre * 9.0)
          cy += 0.030 * (fbm(u * 2.0 + elapsed * 0.035, centre * 4.0, octaves=2) - 0.5)
          band = math.exp(-((v - cy) ** 2) / (thick * thick))
          ray_phase = u * 42.0 + elapsed * 1.15 + centre * 17.0
          rays = 0.56 + 0.24 * math.sin(ray_phase) + 0.20 * math.sin(ray_phase * 0.47)
          vertical = math.exp(-max(0.0, v - cy) * 2.0)
          fade = clamp(1.08 - v * 0.55)
          value += weight * band * rays * vertical * fade
        line.append(clamp(value * contrast))
      grid.append(line)
    return render_field(width, height, grid, options, self)
