import math

from ..core import Animation, clamp, render_field


GOLDEN_ANGLE = math.pi * (3.0 - math.sqrt(5.0))     # 137.5077 degrees


class PhyllotaxisAnimation(Animation):
  """A phyllotactic spiral -- the way real sunflower seeds and pinecone scales
  pack themselves. Each new seed is placed at angle n * (golden angle) with
  radius proportional to sqrt(n), so the seeds tile the disc without gaps. The
  pattern slowly rotates and pulses."""

  thresholds = [
    (0.10, " "), (0.26, "."), (0.42, ":"), (0.56, "-"), (0.68, "+"),
    (0.80, "*"), (0.90, "#"), (1.01, "@"),
  ]

  def render(self, width, height, elapsed, phase, options):
    seeds = int(450 * max(0.5, options.scale))
    ax = width / max(1, height * 2.0)
    cx, cy = width * 0.5, height * 0.5
    # fill the screen: vertical bound dominates because rows are ~2x taller
    radius_world = min(width * 0.5 / max(0.1, ax), height) * 0.95
    scale = radius_world / math.sqrt(seeds)
    rot = elapsed * 0.18
    contrast = options.contrast
    grid = [[0.0] * width for _ in range(height)]
    for n in range(1, seeds + 1):
      r = math.sqrt(n) * scale
      a = n * GOLDEN_ANGLE + rot
      px = cx + r * math.cos(a) * ax
      py = cy + r * math.sin(a) * 0.5
      bright = clamp(0.55 + 0.45 * math.sin(elapsed * 0.55 + n * 0.06))
      ci, ri = int(round(px)), int(round(py))
      if 0 <= ci < width and 0 <= ri < height and grid[ri][ci] < bright:
        grid[ri][ci] = bright
      # soft neighbour so dots read as small petals
      if 0 <= ci + 1 < width and 0 <= ri < height and grid[ri][ci + 1] < bright * 0.6:
        grid[ri][ci + 1] = bright * 0.6
    if contrast != 1.0:
      for r in range(height):
        for c in range(width):
          grid[r][c] = clamp(grid[r][c] * contrast)
    return render_field(width, height, grid, options, self)
