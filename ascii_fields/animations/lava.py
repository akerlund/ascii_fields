import math

from ..core import Animation, clamp, render_field
from ..noise import fbm


class LavaAnimation(Animation):
  """A molten lava surface with convection cells, hot bubbles, dark cooled
  cracks running across the crust, and occasional bursting rings."""

  default_theme = "lava"

  thresholds = [
    (0.07, " "), (0.16, "."), (0.27, ":"), (0.39, "-"), (0.52, "="),
    (0.65, "+"), (0.78, "*"), (0.90, "#"), (1.01, "@"),
  ]

  BUBBLES = (
    (0.18, 0.82, 5.0, 0.00, 0.08),
    (0.34, 0.76, 6.4, 0.23, 0.06),
    (0.56, 0.84, 4.6, 0.47, 0.09),
    (0.74, 0.78, 7.2, 0.68, 0.055),
    (0.88, 0.86, 5.7, 0.81, 0.07),
  )

  def render(self, width, height, elapsed, phase, options):
    contrast = options.contrast
    # Precompute per-bubble state once per frame (was being recomputed per cell).
    bubble_state = []
    for bx, by, period, offset, radius in self.BUBBLES:
      local = (elapsed / period + offset) % 1.0
      rise = by - local * 0.42
      wobble = 0.035 * math.sin(elapsed * 1.3 + offset * 9.0)
      bx_eff = bx + wobble
      bursting = local > 0.78
      burst = (local - 0.78) / 0.22 if bursting else 0.0
      ring_r = 0.45 + burst * 2.1 if bursting else 0.0
      ring_decay = (1.0 - burst) ** 1.5 if bursting else 0.0
      spark_phase = offset * 140.0
      bubble_state.append((bx_eff, rise, radius, local, bursting, ring_r, ring_decay, spark_phase))

    grid = []
    for row in range(height):
      v = row / max(1, height - 1)
      one_minus_v = 1.0 - v
      sin_v7 = math.sin(v * 7.0) * 1.6           # row-only term in cracks
      drift = elapsed * 0.18
      line = []
      for col in range(width):
        u = col / max(1, width - 1)
        convection = fbm(u * 3.0 + elapsed * 0.10, v * 2.4 - drift, octaves=4)
        cracks = abs(math.sin(u * 18.0 + sin_v7 - elapsed * 0.8))
        # base heat with wider contrast and dark cooled crust along cracks
        heat = 0.16 + 0.52 * convection + 0.22 * one_minus_v
        heat -= max(0.0, cracks - 0.55) * 0.40   # cracks are *dark*

        for bx_eff, rise, radius, local, bursting, ring_r, ring_decay, spark_phase in bubble_state:
          inv_r = 1.0 / max(0.001, radius)
          dx = (u - bx_eff) * inv_r
          dy = (v - rise) * inv_r / 0.55
          dist2 = dx * dx + dy * dy
          bubble = math.exp(-dist2 * 1.7)
          heat += bubble * (0.34 + 0.22 * local)
          if bursting:
            dist = math.sqrt(dist2)
            ring = math.exp(-((dist - ring_r) ** 2) / 0.08) * ring_decay
            sparks = max(0.0, math.sin((u + spark_phase * 0.01) * 140.0 + elapsed * 9.0))
            heat += ring * (0.60 + 0.30 * sparks)

        line.append(clamp(heat * contrast))
      grid.append(line)
    return render_field(width, height, grid, options, self)
