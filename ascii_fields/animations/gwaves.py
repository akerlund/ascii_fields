import math

from ..core import Animation, clamp, render_field


CYCLE = 13.0
T_INSPIRAL = 9.5
R_MAX = 0.62
R_MIN = 0.06


class GravitationalWavesAnimation(Animation):
  """Two black holes spiral together, radiating gravitational waves. The
  two-armed quadrupole pattern winds outward and chirps faster as the orbit
  tightens, then a flash marks the merger and the remnant rings down -- looping
  back to a wide, slow inspiral."""

  thresholds = [
    (0.10, " "), (0.20, "."), (0.32, ":"), (0.44, "-"), (0.56, "="),
    (0.68, "+"), (0.80, "*"), (0.90, "#"), (1.01, "@"),
  ]

  def __init__(self):
    self._phi = 0.0
    self._cycle_start = 0.0
    self._last = 0.0

  def render(self, width, height, elapsed, phase, options):
    if elapsed < self._last:
      self._phi = 0.0
      self._cycle_start = elapsed
    local = elapsed - self._cycle_start
    if local > CYCLE:
      self._cycle_start = elapsed
      self._phi = 0.0
      local = 0.0
    dt = clamp(elapsed - self._last, 0.0, 0.1)
    self._last = elapsed

    merging = local >= T_INSPIRAL
    if not merging:
      frac = local / T_INSPIRAL
      sep = R_MAX * (1.0 - frac) ** 0.5 + R_MIN
    else:
      sep = R_MIN
    omega = min(7.0, 0.6 / (sep ** 1.5))
    self._phi += omega * dt

    radius = max(1.0, min(width, height * 2) * 0.5)
    amp = clamp(0.25 + 0.9 * (R_MAX - sep) / R_MAX)
    ring_age = local - T_INSPIRAL
    flash = math.exp(-((ring_age) ** 2) / 0.05) if merging else 0.0
    contrast = options.contrast

    # positions of the two holes
    bx1, by1 = sep * math.cos(self._phi), sep * math.sin(self._phi)
    bx2, by2 = -bx1, -by1

    grid = []
    for row in range(height):
      py = ((row - (height - 1) * 0.5) * 2.0) / radius
      line = []
      for col in range(width):
        px = (col - (width - 1) * 0.5) / radius
        r = math.hypot(px, py)
        a = math.atan2(py, px)
        phi_ret = self._phi - omega * r * 1.6
        strain = amp * math.cos(2.0 * a - 2.0 * phi_ret) / (r + 0.30)
        envelope = math.exp(-r * 0.55) * min(1.0, r * 3.0)
        level = 0.16 + 0.6 * strain * envelope
        if merging:
          rr = ring_age * 1.1
          level += 0.7 * math.exp(-((r - rr) ** 2) / 0.012)
          level += flash * math.exp(-r * 2.0)
        if not merging:
          # the two orbiting holes: dark cores ringed with light
          for bx, by in ((bx1, by1), (bx2, by2)):
            dr = math.hypot(px - bx, py - by)
            level += 0.8 * math.exp(-(dr * dr) / 0.0016)
            level *= 1.0 - 0.9 * math.exp(-(dr * dr) / 0.0006)
        else:
          dr = math.hypot(px, py)
          level += 0.9 * math.exp(-(dr * dr) / 0.004)
          level *= 1.0 - 0.92 * math.exp(-(dr * dr) / 0.0016)
        line.append(clamp(level * contrast))
      grid.append(line)
    return render_field(width, height, grid, options, self)
