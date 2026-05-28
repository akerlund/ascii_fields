import itertools
import math

from ..core import Animation, clamp, render_field


VERTICES = list(itertools.product((-1.0, 1.0), repeat=4))
EDGES = [
  (i, j)
  for i in range(16)
  for j in range(i + 1, 16)
  if sum(1 for a, b in zip(VERTICES[i], VERTICES[j]) if a != b) == 1
]


def _rotate(p, axw, ayz, axy):
  x, y, z, w = p
  c, s = math.cos(axw), math.sin(axw)
  x, w = x * c - w * s, x * s + w * c
  c, s = math.cos(ayz), math.sin(ayz)
  y, z = y * c - z * s, y * s + z * c
  c, s = math.cos(axy), math.sin(axy)
  x, y = x * c - y * s, x * s + y * c
  return x, y, z, w


class HypercubeAnimation(Animation):
  """A tesseract -- a 4D hypercube -- rotating through planes that don't exist
  in 3D, so its inner cube appears to turn itself inside out."""

  thresholds = [
    (0.06, " "), (0.30, "."), (0.45, ":"), (0.58, "-"), (0.70, "+"),
    (0.82, "*"), (0.92, "#"), (1.01, "@"),
  ]

  def render(self, width, height, elapsed, phase, options):
    axw = elapsed * 0.55
    ayz = elapsed * 0.37
    axy = elapsed * 0.20
    gain = min(width, height * 2) * 0.62
    cx, cy = width * 0.5, height * 0.5

    pts = []
    depths = []
    for v in VERTICES:
      x, y, z, w = _rotate(v, axw, ayz, axy)
      k4 = 1.0 / (2.3 - w)
      x, y, z = x * k4, y * k4, z * k4
      k3 = 1.0 / (2.4 - z)
      sx = cx + x * k3 * gain
      sy = cy + y * k3 * gain * 0.5
      pts.append((sx, sy))
      depths.append(k3)
    dmin, dmax = min(depths), max(depths)
    span = (dmax - dmin) or 1.0

    grid = [[0.0] * width for _ in range(height)]
    for i, j in EDGES:
      x0, y0 = pts[i]
      x1, y1 = pts[j]
      b0 = 0.35 + 0.65 * (depths[i] - dmin) / span
      b1 = 0.35 + 0.65 * (depths[j] - dmin) / span
      steps = max(1, int(math.hypot(x1 - x0, y1 - y0)))
      for s in range(steps + 1):
        f = s / steps
        px = int(round(x0 + (x1 - x0) * f))
        py = int(round(y0 + (y1 - y0) * f))
        if 0 <= px < width and 0 <= py < height:
          b = b0 + (b1 - b0) * f
          if b > grid[py][px]:
            grid[py][px] = b
    return render_field(width, height, grid, options, self)
