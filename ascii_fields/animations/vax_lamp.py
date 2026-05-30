import math

from ..core import Animation, clamp, render_field


class VaxLampAnimation(Animation):
  """A lava-lamp-like column of warm wax blobs that stretch, merge and drift
  upward inside a glass tube."""

  default_theme = "lava"

  thresholds = [
    (0.08, " "), (0.18, "."), (0.30, ":"), (0.42, "-"), (0.54, "="),
    (0.66, "+"), (0.78, "*"), (0.90, "#"), (1.01, "@"),
  ]

  BLOBS = (
    (0.42, 0.00, 0.17, 0.20, 8.5),
    (0.58, 0.22, 0.13, 0.16, 6.8),
    (0.48, 0.48, 0.20, 0.24, 10.0),
    (0.62, 0.70, 0.11, 0.15, 5.7),
  )

  def render(self, width, height, elapsed, phase, options):
    ax = width / max(1, height * 2.0)
    contrast = options.contrast
    # Precompute per-blob state once per frame.
    blob_state = []
    for bx, offset, rx, ry, period in self.BLOBS:
      local = (elapsed / period + offset) % 1.0
      cy = 1.15 - local * 2.30
      if cy < -1.15:
        cy += 2.30
      cx0 = (bx - 0.5) * 2.0 * ax
      sway = 0.08 * ax * math.sin(elapsed * 0.55 + offset * 12.0)
      # blobs only stretch vertically while rising past the middle (warming)
      rise_fraction = clamp(1.0 - abs(2.0 * local - 1.0), 0.0, 1.0)
      stretch = 1.0 + 0.45 * rise_fraction
      inv_rx = 1.0 / max(0.001, rx * ax)
      inv_ry = 1.0 / max(0.001, ry * stretch)
      blob_state.append((cx0, sway, cy, inv_rx, inv_ry, offset))

    grid = []
    for row in range(height):
      v = row / max(1, height - 1)
      y = (v - 0.5) * 2.0
      lamp_width = 0.34 * ax * (0.72 + 0.25 * (1.0 - y * y))
      inv_lamp = 1.0 / max(0.001, lamp_width)
      # Cap glow at top and bottom (per-row constant).
      cap = math.exp(-((v - 0.04) ** 2) / 0.002) + math.exp(-((v - 0.96) ** 2) / 0.002)
      line = []
      for col in range(width):
        u = col / max(1, width - 1)
        x = (u - 0.5) * 2.0 * ax
        wall = abs(x) * inv_lamp
        # Brighter glass outline so the tube reads cleanly.
        glass = 0.20 * math.exp(-((wall - 1.0) ** 2) / 0.012)
        value = glass
        if wall < 1.0:
          value += 0.05 + 0.12 * (1.0 - v)
          for cx0, sway, cy, inv_rx, inv_ry, offset in blob_state:
            cx = cx0 + sway * math.sin(y * 3.0 + offset)
            dx = (x - cx) * inv_rx
            dy = (y - cy) * inv_ry
            blob = math.exp(-(dx * dx + dy * dy) * 1.5)
            value += blob * 0.78
          value += cap * 0.26
        line.append(clamp(value * contrast))
      grid.append(line)
    return render_field(width, height, grid, options, self)
