import math

from ..core import Animation, clamp, render_field


class MagneticAnimation(Animation):
  """The dipole field of a bar magnet. Field lines arc from the north pole to
  the south pole and a current of tracers flows along them. The line pattern is
  the streamfunction psi = angle(N) - angle(S); its level sets are the field
  lines."""

  default_theme = "ice"

  thresholds = [
    (0.18, " "), (0.30, "."), (0.42, ":"), (0.54, "-"), (0.66, "="),
    (0.76, "+"), (0.85, "*"), (0.93, "#"), (1.01, "@"),
  ]

  def render(self, width, height, elapsed, phase, options):
    flow = elapsed * 2.2
    lines = 9.0
    d = 0.34
    ax = width / max(1, height * 2.0)
    contrast = options.contrast
    grid = []
    for row in range(height):
      py = (row / max(1, height - 1) - 0.5) * 2.0
      line = []
      for col in range(width):
        px = (col / max(1, width - 1) - 0.5) * 2.0 * ax
        # streamfunction of two opposite poles on the x-axis
        psi = math.atan2(py, px - d) - math.atan2(py, px + d)
        field_lines = 0.5 + 0.5 * math.sin(lines * psi - flow)
        # field strength (dipole) brightens lines near the poles
        rn = math.hypot(px - d, py) + 0.05
        rs = math.hypot(px + d, py) + 0.05
        strength = clamp(0.10 + 0.65 / (rn * rn) + 0.65 / (rs * rs), 0.0, 1.2)
        level = field_lines * strength
        # the magnet body and pole caps
        if abs(py) < 0.14 and abs(px) < d + 0.06:
          level = 0.95 if px > 0 else 0.6
        line.append(clamp(level * contrast))
      grid.append(line)
    return render_field(width, height, grid, options, self)
