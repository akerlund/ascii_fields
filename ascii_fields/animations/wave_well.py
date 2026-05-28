import math

from ..core import Animation, clamp, render_field


class WaveWellAnimation(Animation):
  """A quantum wave packet trapped in a 2D harmonic well. |psi|^2 is built from
  a few energy eigenstates with different phases, so the packet sloshes, spreads
  and periodically revives -- with the parabolic well walls drawn faintly."""

  thresholds = [
    (0.10, " "), (0.20, "."), (0.32, ":"), (0.44, "-"), (0.56, "="),
    (0.68, "+"), (0.80, "*"), (0.90, "#"), (1.01, "@"),
  ]

  # superposition of coherent-state lobes: (amplitude, energy, x-phase, y-phase)
  MODES = (
    (1.00, 1.0, 0.0, 0.0),
    (0.70, 2.0, 0.6, 1.1),
    (0.45, 3.0, 1.7, 0.3),
  )

  def render(self, width, height, elapsed, phase, options):
    t = elapsed * 1.3
    # packet centre traces a Lissajous path inside the well
    cx = 0.42 * math.sin(t)
    cy = 0.30 * math.sin(1.7 * t + 0.5)
    sigma = 0.30 + 0.10 * math.sin(t * 0.8)
    contrast = options.contrast
    grid = []
    for row in range(height):
      y = (row / max(1, height - 1) - 0.5) * 2.0
      line = []
      for col in range(width):
        x = (col / max(1, width - 1) - 0.5) * 2.0
        well = x * x + y * y
        # real and imaginary parts of the packet superposition
        re = 0.0
        im = 0.0
        for amp, energy, phx, phy in self.MODES:
          envelope = math.exp(-((x - cx) ** 2 + (y - cy) ** 2) / (sigma * sigma))
          ripple = math.cos(6.0 * (x * math.cos(phx) + y * math.sin(phy)) - energy * t)
          rippl2 = math.sin(6.0 * (x * math.cos(phx) + y * math.sin(phy)) - energy * t)
          re += amp * envelope * ripple
          im += amp * envelope * rippl2
        prob = re * re + im * im
        wall = 0.06 * math.exp(-((math.sqrt(well) - 0.95) ** 2) / 0.01)
        level = clamp(prob * 0.5 * contrast + wall)
        line.append(level)
      grid.append(line)
    return render_field(width, height, grid, options, self)
