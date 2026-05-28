import math

from ..core import Animation, clamp, render_field


def _fibonacci_sphere(n):
  pts = []
  ga = math.pi * (3.0 - math.sqrt(5.0))
  for i in range(n):
    y = 1.0 - (i / (n - 1)) * 2.0
    r = math.sqrt(max(0.0, 1.0 - y * y))
    theta = ga * i
    pts.append((math.cos(theta) * r, y, math.sin(theta) * r))
  return pts


ATOMS = _fibonacci_sphere(42)
# bonds: connect each atom to its nearest neighbours (cage edges)
_BONDS = []
for _i in range(len(ATOMS)):
  dists = sorted(
    range(len(ATOMS)),
    key=lambda _j, _i=_i: (ATOMS[_i][0] - ATOMS[_j][0]) ** 2
    + (ATOMS[_i][1] - ATOMS[_j][1]) ** 2 + (ATOMS[_i][2] - ATOMS[_j][2]) ** 2,
  )
  for _j in dists[1:4]:
    edge = (min(_i, _j), max(_i, _j))
    if edge not in _BONDS:
      _BONDS.append(edge)


class MoleculeAnimation(Animation):
  """A rotating molecular cage -- a fullerene-like ball of atoms joined by
  bonds, tumbling in 3D. Atoms and bonds nearer the viewer are drawn brighter
  for depth."""

  default_theme = "nebula"

  thresholds = [
    (0.06, " "), (0.26, "."), (0.42, ":"), (0.56, "-"), (0.68, "+"),
    (0.80, "*"), (0.90, "#"), (1.01, "@"),
  ]

  def render(self, width, height, elapsed, phase, options):
    ay = elapsed * 0.6
    ax = elapsed * 0.37
    cay, say = math.cos(ay), math.sin(ay)
    cax, sax = math.cos(ax), math.sin(ax)
    gain = min(width, height * 2) * 0.55
    cx, cy = width * 0.5, height * 0.5

    pts = []
    for (x, y, z) in ATOMS:
      x, z = x * cay - z * say, x * say + z * cay
      y, z = y * cax - z * sax, y * sax + z * cax
      k = 1.0 / (2.6 - z)
      pts.append((cx + x * k * gain, cy + y * k * gain * 0.5, z))

    grid = [[0.0] * width for _ in range(height)]

    def splat(col, row, value):
      c, r = int(round(col)), int(round(row))
      if 0 <= c < width and 0 <= r < height and grid[r][c] < value:
        grid[r][c] = value

    for (i, j) in _BONDS:
      x0, y0, z0 = pts[i]
      x1, y1, z1 = pts[j]
      steps = max(1, int(math.hypot(x1 - x0, y1 - y0)))
      for s in range(steps + 1):
        f = s / steps
        z = z0 + (z1 - z0) * f
        splat(x0 + (x1 - x0) * f, y0 + (y1 - y0) * f, 0.30 + 0.30 * (z * 0.5 + 0.5))

    for (x, y, z) in pts:
      bright = 0.55 + 0.45 * (z * 0.5 + 0.5)
      splat(x, y, bright)
      splat(x + 1, y, bright * 0.8)
      splat(x, y + 1, bright * 0.7)
    return render_field(width, height, grid, options, self)
