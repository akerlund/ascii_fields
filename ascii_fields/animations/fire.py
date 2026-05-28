import math

from ..core import Animation, clamp, render_field
from ..noise import fbm


class FireAnimation(Animation):
  """A campfire: heat rises from a hot base into flickering tongues that cool
  and break up as they climb."""

  default_theme = "fire"   # mono by default; --theme fire for real flames

  thresholds = [
    (0.10, " "), (0.20, "."), (0.32, ":"), (0.44, "-"), (0.56, "="),
    (0.68, "+"), (0.80, "*"), (0.90, "#"), (1.01, "@"),
  ]

  def render(self, width, height, elapsed, phase, options):
    contrast = options.contrast
    grid = []
    for row in range(height):
      v = row / max(1, height - 1)
      rise = 1.0 - v                      # 1 at the bottom, 0 at the top
      line = []
      for col in range(width):
        u = col / max(1, width - 1)
        # hot core narrows toward the top; flames lean with low-frequency sway
        sway = 0.10 * math.sin(v * 4.0 - elapsed * 1.5)
        spread = 0.34 + 0.18 * rise
        horiz = math.exp(-((u - 0.5 - sway) ** 2) / (spread * spread))
        turbulence = fbm(u * 5.0, v * 6.0 - elapsed * 4.2, octaves=4)
        flame = (rise ** 0.6) * horiz * (0.35 + 1.15 * turbulence)
        flame -= 0.55 * v                  # cool with height
        flicker = 0.85 + 0.15 * math.sin(elapsed * 9.0 + col * 0.5)
        line.append(clamp(flame * flicker * 1.7 * contrast))
      grid.append(line)
    return render_field(width, height, grid, options, self)
