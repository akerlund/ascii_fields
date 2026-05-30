import math

from ..core import Animation, clamp, render_field


class MagneticAnimation(Animation):
  """The dipole field of a bar magnet -- just the field lines, no drawn magnet.
  Field lines arc from N to S and a current of tracers flows along them. The
  pattern is the streamfunction psi = angle(N) - angle(S); its level sets are
  the lines, and a gentle |B|-like falloff keeps them readable everywhere."""

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
        # field strength: gentle |B|-like falloff, capped so the poles don't
        # paint a saturated 'bar magnet' blob between them
        rn = math.hypot(px - d, py) + 0.18
        rs = math.hypot(px + d, py) + 0.18
        strength = clamp(0.18 + 0.22 / rn + 0.22 / rs, 0.0, 0.95)
        # darken the immediate neighbourhood of each pole so they don't blob
        pole_cut = max(math.exp(-(rn * rn) * 18.0), math.exp(-(rs * rs) * 18.0))
        level = field_lines * strength * (1.0 - 0.85 * pole_cut)
        line.append(clamp(level * contrast))
      grid.append(line)
    return render_field(width, height, grid, options, self)
