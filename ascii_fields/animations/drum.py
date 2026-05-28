import math

from ..core import Animation, clamp, render_field


# A few (n, m, z_{n,m}) eigenmodes of a circular drum -- n angular nodes,
# m radial zero index, z is the m-th positive zero of J_n.
DRUM_MODES = (
  (0, 1, 2.4048),
  (1, 1, 3.8317),
  (2, 1, 5.1356),
  (0, 2, 5.5201),
  (3, 1, 6.3802),
  (1, 2, 7.0156),
  (4, 1, 7.5883),
  (2, 2, 8.4172),
  (0, 3, 8.6537),
)
MODE_SECONDS = 4.0
RADIAL_SAMPLES = 256


def bessel_j(n, x):
  """Bessel function J_n(x) via the standard series. Accurate enough for the
  drum-radius range we render (x < ~12) with a generous term budget."""
  if x == 0.0:
    return 1.0 if n == 0 else 0.0
  half = x * 0.5
  fact_n = math.factorial(n)
  term = (half ** n) / fact_n
  total = term
  sign = -1.0
  k = 1
  while k < 80:
    term *= (half * half) / (k * (k + n))
    delta = sign * term
    total += delta
    if abs(delta) < 1e-9:
      break
    sign = -sign
    k += 1
  return total


class DrumAnimation(Animation):
  """The first few vibrational eigenmodes of a circular drum. Each mode (n, m)
  is a Bessel-radial profile combined with n angular nodes, oscillating at its
  own frequency. The driving slowly steps through modes so the pattern blooms
  into rings, then crosses, then nested rings, and so on."""

  thresholds = [
    (0.10, " "), (0.22, "."), (0.34, ":"), (0.46, "-"), (0.58, "="),
    (0.70, "+"), (0.82, "*"), (0.92, "#"), (1.01, "@"),
  ]

  def __init__(self):
    self._profile = {}

  def _radial(self, n, zero):
    key = (n, zero)
    profile = self._profile.get(key)
    if profile is None:
      profile = [bessel_j(n, zero * (i / (RADIAL_SAMPLES - 1))) for i in range(RADIAL_SAMPLES)]
      peak = max(abs(v) for v in profile) or 1.0
      profile = [v / peak for v in profile]
      self._profile[key] = profile
    return profile

  def render(self, width, height, elapsed, phase, options):
    idx = int(elapsed / MODE_SECONDS) % len(DRUM_MODES)
    n, m, zero = DRUM_MODES[idx]
    profile = self._radial(n, zero)
    omega = math.sqrt(zero) * 0.9
    # Keep the spatial pattern visible at every phase: pulse without zeroing.
    pulse = 0.72 + 0.28 * math.cos(omega * elapsed)
    ax = width / max(1, height * 2.0)
    contrast = options.contrast
    last_idx = RADIAL_SAMPLES - 1
    grid = []
    for row in range(height):
      py = (row / max(1, height - 1) - 0.5) * 2.0
      line = []
      for col in range(width):
        px = (col / max(1, width - 1) - 0.5) * 2.0 * ax
        r = math.hypot(px, py)
        if r >= 1.0:
          line.append(0.0)
          continue
        radial = profile[int(r * last_idx)]
        u = radial * math.cos(n * math.atan2(py, px))
        # render |u| so the standing wave reads as bright crests + dark nodes
        line.append(clamp(abs(u) * pulse * 1.4 * contrast))
      grid.append(line)
    return render_field(width, height, grid, options, self)
