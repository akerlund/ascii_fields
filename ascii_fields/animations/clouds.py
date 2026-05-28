import math

from ..core import Animation, clamp, render_field
from ..noise import fbm


class CloudsAnimation(Animation):
  """Soft fractal-noise clouds drifting across the sky, with a brighter horizon
  glow low on the screen. Domain warping keeps the billows from looking too
  regular."""

  thresholds = [
    (0.18, " "), (0.30, "."), (0.42, ":"), (0.52, "-"), (0.62, "="),
    (0.72, "+"), (0.82, "*"), (0.92, "#"), (1.01, "@"),
  ]

  def render(self, width, height, elapsed, phase, options):
    drift = elapsed * 0.06
    freq = 3.2 * max(0.4, options.scale)
    contrast = options.contrast
    grid = []
    for row in range(height):
      v = row / max(1, height - 1)
      glow = 0.12 + 0.18 * v                # brighter near the horizon
      line = []
      for col in range(width):
        u = col / max(1, width - 1)
        # domain warp for puffier billows
        wx = fbm(u * freq + drift, v * freq, octaves=3)
        wy = fbm(u * freq + 5.2, v * freq - drift * 0.6, octaves=3)
        n = fbm(u * freq + drift + wx * 1.5, v * freq * 1.4 + wy * 1.5, octaves=5)
        cloud = max(0.0, n - 0.42) / 0.58
        level = glow + cloud * 0.95
        line.append(clamp(level * contrast))
      grid.append(line)
    return render_field(width, height, grid, options, self)
