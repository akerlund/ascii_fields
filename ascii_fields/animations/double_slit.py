import math

from ..core import Animation, clamp, render_field


class DoubleSlitAnimation(Animation):
  """The double-slit experiment. A plane wave hits a barrier with two slits;
  the two emerging circular waves interfere, and a detector strip on the right
  slowly accumulates the characteristic fringe pattern."""

  thresholds = [
    (0.10, " "), (0.20, "."), (0.32, ":"), (0.44, "-"), (0.56, "="),
    (0.68, "+"), (0.80, "*"), (0.90, "#"), (1.01, "@"),
  ]

  def __init__(self):
    self._screen = None        # accumulated detector intensity per row
    self._h = 0
    self._last_elapsed = 0.0

  def render(self, width, height, elapsed, phase, options):
    if self._screen is None or height != self._h or elapsed < self._last_elapsed:
      self._screen = [0.0] * height
      self._h = height
    dt = max(0.0, elapsed - self._last_elapsed)
    self._last_elapsed = elapsed

    barrier_x = 0.32
    slit_y1, slit_y2 = 0.40, 0.60
    k = 42.0
    omega = 7.0
    aspect = width / max(1, height * 2.0)
    detector_x = 0.93

    # geometry of the two slits in normalised, aspect-corrected coords
    s1x, s1y = barrier_x * aspect, slit_y1
    s2x, s2y = barrier_x * aspect, slit_y2

    # accumulate the detector pattern from the current interference intensity
    peak = 0.0
    new_screen = []
    for row in range(height):
      v = row / max(1, height - 1)
      dx = detector_x * aspect
      r1 = math.hypot(dx - s1x, v - s1y)
      r2 = math.hypot(dx - s2x, v - s2y)
      amp = math.sin(k * r1) + math.sin(k * r2)
      inten = amp * amp
      acc = self._screen[row] + inten * dt * 0.18
      new_screen.append(acc)
      peak = max(peak, acc)
    self._screen = new_screen
    norm = 1.0 / peak if peak > 1e-6 else 0.0

    grid = []
    for row in range(height):
      v = row / max(1, height - 1)
      fringe = self._screen[row] * norm
      line = []
      for col in range(width):
        u = col / max(1, width - 1)
        x = u * aspect
        if u < barrier_x:
          # incoming plane wave travelling right
          level = 0.30 + 0.30 * math.sin(k * x - omega * elapsed)
        elif u < barrier_x + 0.012:
          # barrier, with the two slits cut out
          near_slit = min(abs(v - slit_y1), abs(v - slit_y2))
          level = 0.0 if near_slit < 0.035 else 0.92
        elif u > detector_x:
          # detector screen building up fringes
          level = 0.12 + 0.88 * fringe
        else:
          r1 = math.hypot(x - s1x, v - s1y)
          r2 = math.hypot(x - s2x, v - s2y)
          amp = (math.sin(k * r1 - omega * elapsed) / (0.4 + r1)
                 + math.sin(k * r2 - omega * elapsed) / (0.4 + r2))
          level = 0.32 + amp * 0.5
        line.append(clamp(level * options.contrast))
      grid.append(line)
    return render_field(width, height, grid, options, self)
