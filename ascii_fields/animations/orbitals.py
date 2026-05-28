import math

from ..core import Animation, clamp, render_field


def _1s(x, z, r):
  return math.exp(-r)


def _2s(x, z, r):
  return (1.0 - 0.5 * r) * math.exp(-0.5 * r)


def _2pz(x, z, r):
  return z * math.exp(-0.5 * r)


def _3dz2(x, z, r):
  return (3.0 * z * z - r * r) * math.exp(-r / 3.0)


def _3dxz(x, z, r):
  return x * z * math.exp(-r / 3.0)


def _4fz3(x, z, r):
  return z * (5.0 * z * z - 3.0 * r * r) * math.exp(-0.25 * r)


# (label, amplitude fn, world half-extent in Bohr radii)
ORBITALS = [
  ("1s", _1s, 6.0),
  ("2p", _2pz, 12.0),
  ("2s", _2s, 14.0),
  ("3d_z2", _3dz2, 20.0),
  ("3d_xz", _3dxz, 20.0),
  ("4f", _4fz3, 26.0),
]

STATE_SECONDS = 4.5
TRANSITION = 1.6


class OrbitalsAnimation(Animation):
  """Hydrogen electron probability clouds |psi|^2 in a cross-section, slowly
  morphing through real (n, l, m) orbitals: 1s -> 2p -> 2s -> 3d -> 4f ..."""

  thresholds = [
    (0.05, " "), (0.14, "."), (0.26, ":"), (0.40, "-"), (0.54, "="),
    (0.68, "+"), (0.80, "*"), (0.90, "#"), (1.01, "@"),
  ]

  def __init__(self):
    self._norm = {}

  def _normalizer(self, index):
    if index not in self._norm:
      _, fn, extent = ORBITALS[index]
      peak = 1e-9
      for i in range(41):
        for j in range(41):
          x = (i / 40.0 - 0.5) * 2.0 * extent
          z = (j / 40.0 - 0.5) * 2.0 * extent
          a = fn(x, z, math.hypot(x, z))
          peak = max(peak, a * a)
      self._norm[index] = 1.0 / peak
    return self._norm[index]

  def render(self, width, height, elapsed, phase, options):
    cycle = STATE_SECONDS
    pos = elapsed / cycle
    i0 = int(pos) % len(ORBITALS)
    i1 = (i0 + 1) % len(ORBITALS)
    frac = pos - math.floor(pos)
    blend = clamp((frac - (1.0 - TRANSITION / cycle)) / (TRANSITION / cycle))

    label0, fn0, ext0 = ORBITALS[i0]
    label1, fn1, ext1 = ORBITALS[i1]
    n0 = self._normalizer(i0)
    n1 = self._normalizer(i1)
    contrast = options.contrast
    grid = []
    for row in range(height):
      pz = (row - (height - 1) * 0.5) / max(1.0, height * 0.5)
      line = []
      for col in range(width):
        px = (col - (width - 1) * 0.5) / max(1.0, width * 0.5)
        x0, z0 = px * ext0, pz * ext0
        a0 = fn0(x0, z0, math.hypot(x0, z0))
        d0 = a0 * a0 * n0
        if blend > 0.0:
          x1, z1 = px * ext1, pz * ext1
          a1 = fn1(x1, z1, math.hypot(x1, z1))
          d1 = a1 * a1 * n1
          d = d0 * (1.0 - blend) + d1 * blend
        else:
          d = d0
        level = (clamp(d)) ** 0.45
        line.append(clamp(level * contrast))
      grid.append(line)
    return render_field(width, height, grid, options, self)
