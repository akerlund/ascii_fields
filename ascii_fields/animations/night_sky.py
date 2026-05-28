import math

from ..core import Animation, clamp, render_field, star_noise
from ..noise import fbm


class NightSkyAnimation(Animation):
  """A still night sky: twinkling stars, a soft Milky Way band, and the odd
  shooting star streaking across."""

  thresholds = [
    (0.06, " "), (0.16, "."), (0.30, ":"), (0.45, "-"), (0.60, "+"),
    (0.74, "*"), (0.86, "o"), (0.94, "#"), (1.01, "@"),
  ]

  def render(self, width, height, elapsed, phase, options):
    density = max(0.4, options.scale)
    # Milky-way band: a diagonal swathe of faint fbm haze.
    band_angle = 0.5
    ca, sa = math.cos(band_angle), math.sin(band_angle)
    meteor = self._meteor(elapsed, width, height)
    grid = []
    for row in range(height):
      ny = (row / max(1, height - 1) - 0.5) * 2.0
      line = []
      for col in range(width):
        nx = (col / max(1, width - 1) - 0.5) * 2.0
        # distance from the band centre line, rotated
        across = nx * sa - ny * ca
        haze = math.exp(-(across * across) / 0.12)
        milky = haze * (0.10 + 0.26 * fbm(nx * 3.0 + 4.0, ny * 3.0, octaves=4, seed=7))
        level = 0.04 + milky
        # Stars: sparse, twinkling.
        n = star_noise(col, row)
        if n > 0.991:
          mag = 0.55 + 0.45 * star_noise(col + 3, row + 11)
          twinkle = 0.6 + 0.4 * math.sin(elapsed * (1.5 + 4.0 * n) + col * 0.7 + row)
          level = max(level, mag * twinkle * (0.7 + 0.3 * density))
        elif n > 0.975:
          level = max(level, 0.22 + 0.10 * math.sin(elapsed * 2.0 + col))
        if meteor is not None:
          level = max(level, self._meteor_brightness(meteor, col, row))
        line.append(clamp(level))
      grid.append(line)
    return render_field(width, height, grid, options, self)

  def _meteor(self, elapsed, width, height):
    period = 6.5
    idx = int(elapsed / period)
    local = elapsed - idx * period
    duration = 1.1
    if local > duration:
      return None
    r = star_noise(idx * 17 + 3, idx * 5 + 1)
    r2 = star_noise(idx * 9 + 7, idx * 13 + 2)
    start_x = r * width
    start_y = r2 * height * 0.5
    dx = (0.7 + 0.5 * r2) * width
    dy = (0.4 + 0.4 * r) * height
    p = local / duration
    head_x = start_x + dx * p
    head_y = start_y + dy * p
    return (head_x, head_y, dx, dy, 1.0 - p)

  def _meteor_brightness(self, meteor, col, row):
    head_x, head_y, dx, dy, fade = meteor
    length = math.hypot(dx, dy)
    ux, uy = dx / length, dy / length
    rx, ry = col - head_x, row - head_y
    along = rx * ux + ry * uy            # negative = behind head (the tail)
    perp = abs(rx * uy - ry * ux)
    if along > 0.6 or along < -10.0 or perp > 1.4:
      return 0.0
    tail = math.exp(along * 0.45) if along < 0 else 1.0
    core = math.exp(-(perp * perp) / 0.5)
    return clamp(tail * core * fade * 1.2)
