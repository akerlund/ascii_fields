import math

from ..core import Animation, clamp, render_field
from ..noise import fbm


class JupiterStormAnimation(Animation):
  """Jupiter's Great Red Spot as a layered banded atmosphere with a rotating
  oval vortex, a dark eye, and turbulent streamers around it."""

  default_theme = "sunset"

  thresholds = [
    (0.08, " "), (0.18, "."), (0.30, ":"), (0.42, "-"), (0.54, "="),
    (0.66, "+"), (0.78, "*"), (0.90, "#"), (1.01, "@"),
  ]

  def render(self, width, height, elapsed, phase, options):
    ax = width / max(1, height * 2.0)
    cx = 0.08 * ax * math.sin(elapsed * 0.12)
    cy = 0.03 * math.sin(elapsed * 0.18)
    spin = elapsed * 0.55
    contrast = options.contrast
    # bands drift horizontally so the atmosphere is alive instead of static
    band_drift = elapsed * 0.06
    cos_spin = math.cos(spin)
    sin_spin = math.sin(spin)
    inv_rx = 1.0 / max(0.22, 0.36 * ax)
    grid = []

    for row in range(height):
      y = (row / max(1, height - 1) - 0.5) * 2.0
      # per-row band invariants (cheap row-only sines hoisted out of inner loop)
      band_base = 0.10 + 0.13 * math.sin(y * 18.0 + elapsed * 0.25)
      band_base += 0.07 * math.sin(y * 43.0 - elapsed * 0.15)
      shear = 0.12 * math.sin(y * 9.0 + elapsed * 0.2)
      wind_freq_x = 1.8 * (0.10 + 0.08 * math.sin(y * 7.0)) + 1.0  # constant per row
      dy = (y - cy) / 0.25
      line = []
      for col in range(width):
        x = (col / max(1, width - 1) - 0.5) * 2.0 * ax
        wind = fbm(x * 1.8 + band_drift, y * 5.0 + shear, octaves=3)
        value = band_base + 0.18 * wind

        dx = (x - cx) * inv_rx
        r = math.hypot(dx, dy)
        theta = math.atan2(dy, dx)
        oval = math.exp(-(r * r) * 1.35)
        wall = math.exp(-((r - 0.95) ** 2) / 0.028)
        eye = math.exp(-(r * r) / 0.12)
        swirl = 0.5 + 0.5 * math.sin(theta * 4.0 - spin + r * 8.5)
        turbulent = fbm(dx * 2.2 + cos_spin * 0.4, dy * 2.2 + sin_spin * 0.4, octaves=3)
        streamer = math.exp(-((abs(dy) - 0.75) ** 2) / 0.10)
        value += oval * (0.25 + 0.30 * swirl + 0.20 * turbulent)
        value += wall * (0.42 + 0.22 * swirl)
        value -= eye * 0.55              # dark eye of the storm
        value += streamer * max(0.0, 1.0 - abs(dx) * 0.9) * 0.18
        line.append(clamp(value * contrast))
      grid.append(line)
    return render_field(width, height, grid, options, self)
