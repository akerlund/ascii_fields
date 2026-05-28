import math

from ..core import Animation, clamp, render_field


# Chladni mode pairs for a square plate (free edges, classical sin sin difference).
MODES = ((2, 3), (3, 4), (4, 5), (5, 6), (4, 7), (3, 5), (6, 7), (2, 5), (5, 3), (7, 4))
MODE_SECONDS = 4.5
TRANSITION = 1.4


class ChladniAnimation(Animation):
  """A square plate vibrated at one of its eigenfrequencies. The standing wave
  u(x, y) = sin(m pi x) sin(n pi y) - sin(n pi x) sin(m pi y) has nodal lines
  where the plate isn't moving -- sand piles up there. The driving frequency
  slowly steps through the (m, n) modes, drawing different patterns."""

  thresholds = [
    (0.10, " "), (0.22, "."), (0.34, ":"), (0.46, "-"), (0.58, "="),
    (0.70, "+"), (0.82, "*"), (0.92, "#"), (1.01, "@"),
  ]

  def render(self, width, height, elapsed, phase, options):
    idx = int(elapsed / MODE_SECONDS) % len(MODES)
    nxt = (idx + 1) % len(MODES)
    frac = (elapsed % MODE_SECONDS) / MODE_SECONDS
    blend = clamp((frac - (1.0 - TRANSITION / MODE_SECONDS)) / (TRANSITION / MODE_SECONDS))
    m0, n0 = MODES[idx]
    m1, n1 = MODES[nxt]
    omega = 1.6
    # The sand sits at nodal lines regardless of how hard the plate is shaken,
    # so the *pattern* is amplitude-independent; the drive only pulses the
    # overall brightness slightly so the scene feels alive.
    pulse = 0.78 + 0.22 * abs(math.cos(omega * elapsed))
    contrast = options.contrast
    grid = []
    for row in range(height):
      y = row / max(1, height - 1)
      sin0y_m0 = math.sin(m0 * math.pi * y)
      sin0y_n0 = math.sin(n0 * math.pi * y)
      sin1y_m1 = math.sin(m1 * math.pi * y)
      sin1y_n1 = math.sin(n1 * math.pi * y)
      line = []
      for col in range(width):
        x = col / max(1, width - 1)
        u0 = math.sin(m0 * math.pi * x) * sin0y_n0 - math.sin(n0 * math.pi * x) * sin0y_m0
        u1 = math.sin(m1 * math.pi * x) * sin1y_n1 - math.sin(n1 * math.pi * x) * sin1y_m1
        u = u0 * (1.0 - blend) + u1 * blend
        # sand collects at the nodal lines |u| = 0; sharper exp -> tighter lines
        nodal = math.exp(-abs(u) * 6.0)
        line.append(clamp(nodal * pulse * contrast))
      grid.append(line)
    return render_field(width, height, grid, options, self)
