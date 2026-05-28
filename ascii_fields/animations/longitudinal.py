import math

from ..core import Animation, clamp, render_field


class LongitudinalAnimation(Animation):
  """A longitudinal (compression) wave, like sound. A lattice of particles
  oscillates back and forth along the direction of travel, so they bunch into
  moving compressions and spread into rarefactions. A faint density band
  underlays the dots."""

  thresholds = [
    (0.12, " "), (0.22, "."), (0.34, ":"), (0.46, "-"), (0.58, "="),
    (0.70, "+"), (0.82, "*"), (0.92, "#"), (1.01, "@"),
  ]

  def render(self, width, height, elapsed, phase, options):
    freq = 3.0 * max(0.4, options.scale)     # wavelengths across the screen
    omega = 2.2
    amp = 0.55 / max(1.0, freq)               # displacement amplitude (in wavelengths)
    k = 2.0 * math.pi * freq
    contrast = options.contrast

    grid = [[0.0] * width for _ in range(height)]
    # faint density band: compression brightens the background
    for row in range(height):
      for col in range(width):
        x = col / max(1, width - 1)
        density = 0.5 + 0.5 * math.cos(k * x - omega * elapsed)
        grid[row][col] = 0.10 + 0.16 * density * density

    # particle lattice, displaced longitudinally (horizontally)
    cols = max(12, int(freq * 10))
    row_step = 2 if height >= 10 else 1
    for ix in range(cols):
      x0 = ix / cols
      dx = amp * math.sin(k * x0 - omega * elapsed)
      sx = (x0 + dx)
      col = int(round(sx * (width - 1)))
      if 0 <= col < width:
        for row in range(0, height, row_step):
          if grid[row][col] < 1.0:
            grid[row][col] = 1.0
          # soften neighbours so dots read as small blobs
          if col + 1 < width:
            grid[row][col + 1] = max(grid[row][col + 1], 0.55)
    return render_field(width, height, grid, options, self)
