import math

from ..core import Animation, clamp, render_glyph_field


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
    glyphs = [[" "] * width for _ in range(height)]
    mid = (height - 1) * 0.5
    tube = max(1.0, height * 0.32)

    # Moving density tube: compressions glow through the middle instead of
    # flooding the whole screen evenly.
    for row in range(height):
      yfade = math.exp(-(((row - mid) / tube) ** 2))
      for col in range(width):
        x = col / max(1, width - 1)
        density = 0.5 + 0.5 * math.cos(k * x - omega * elapsed)
        overtone = 0.5 + 0.5 * math.cos(k * 0.52 * x - omega * elapsed * 0.63 + 1.7)
        grid[row][col] = (0.04 + 0.25 * density * density + 0.08 * overtone) * yfade
        if grid[row][col] > 0.10:
          glyphs[row][col] = "-" if density < 0.7 else "="

    # particle lattice, displaced longitudinally (horizontally)
    cols = max(16, int(freq * 14))
    rows = max(3, min(height, 9))
    for ix in range(cols):
      x0 = ix / cols
      dx = amp * math.sin(k * x0 - omega * elapsed)
      sx = (x0 + dx)
      col = int(round(sx * (width - 1)))
      if 0 <= col < width:
        phase_offset = 0.7 * math.sin(ix * 1.9 + elapsed * 1.4)
        for iy in range(rows):
          y0 = (iy + 0.5) / rows
          wobble = 0.08 * math.sin(k * x0 - omega * elapsed + iy * 0.9)
          row = int(round((y0 + wobble - 0.5) * height * 0.62 + mid + phase_offset))
          if 0 <= row < height:
            compression = 0.5 + 0.5 * math.cos(k * x0 - omega * elapsed)
            value = 0.60 + 0.40 * compression
            if value >= grid[row][col]:
              grid[row][col] = value
              glyphs[row][col] = "o" if compression < 0.68 else "@"
            if col + 1 < width:
              grid[row][col + 1] = max(grid[row][col + 1], value * 0.42)
              if glyphs[row][col + 1] == " ":
                glyphs[row][col + 1] = "."
    return render_glyph_field(width, height,
                              [[clamp(v * contrast) for v in row] for row in grid],
                              glyphs, options, self)
