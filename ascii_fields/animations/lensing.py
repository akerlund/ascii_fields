import math

from ..core import Animation, clamp, render_field, star_noise
from ..noise import fbm


def _starfield(bx, by, density):
  """Sample a deterministic point-source background in source-plane coords."""
  ix0 = int(math.floor(bx * density))
  iy0 = int(math.floor(by * density))
  best = 0.0
  for di in (-1, 0, 1):
    for dj in (-1, 0, 1):
      ix, iy = ix0 + di, iy0 + dj
      if star_noise(ix, iy) <= 0.92:
        continue
      sx = (ix + star_noise(ix + 101, iy + 7)) / density
      sy = (iy + star_noise(ix + 19, iy + 31)) / density
      dx = bx - sx
      dy = by - sy
      mag = 0.5 + 0.5 * star_noise(ix + 57, iy + 89)
      val = mag * math.exp(-(dx * dx + dy * dy) * density * density * 6.0)
      if val > best:
        best = val
  return best


class LensingAnimation(Animation):
  """An invisible massive object drifts across a star field, bending the light
  behind it. Background stars stretch into arcs and pop into bright Einstein
  rings as the lens passes directly behind them -- gravitational microlensing
  in motion."""

  thresholds = [
    (0.06, " "), (0.18, "."), (0.30, ":"), (0.42, "-"), (0.54, "="),
    (0.66, "+"), (0.78, "*"), (0.90, "#"), (1.01, "@"),
  ]

  EINSTEIN_R = 0.28          # Einstein radius in world units

  def render(self, width, height, elapsed, phase, options):
    ax = width / max(1, height * 2.0)
    density = 18.0 * max(0.5, options.scale)
    t = elapsed
    lx = 0.95 * ax * math.sin(t * 0.18)
    ly = 0.30 * math.sin(t * 0.27)
    rE = self.EINSTEIN_R
    rE2 = rE * rE
    contrast = options.contrast
    grid = []
    for row in range(height):
      py = (row / max(1, height - 1) - 0.5) * 2.0
      line = []
      for col in range(width):
        px = (col / max(1, width - 1) - 0.5) * 2.0 * ax
        # vector from lens to image
        dxc = px - lx
        dyc = py - ly
        r2 = dxc * dxc + dyc * dyc + 1e-4
        # point-lens equation: source position behind the image
        factor = rE2 / r2
        bx = px - dxc * factor
        by = py - dyc * factor
        star = _starfield(bx, by, density)
        # the Einstein ring is the locus where the image is most magnified
        ring_r = math.sqrt(r2)
        ring = math.exp(-((ring_r - rE) ** 2) / 0.0006) * 0.45
        haze = 0.04 + 0.05 * fbm(bx * 2.4, by * 2.4, octaves=3)
        level = star * (1.0 + 1.0 * ring) + ring * 0.22 + haze
        # subtle dark hint at the lens core
        level *= 1.0 - 0.55 * math.exp(-r2 / 0.0014)
        line.append(clamp(level * contrast))
      grid.append(line)
    return render_field(width, height, grid, options, self)
