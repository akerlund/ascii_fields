import math

from ..core import Animation, clamp, render_field


class DNAAnimation(Animation):
  """A rotating DNA double helix. Two sugar-phosphate backbones wind around each
  other as sine waves a half-turn apart; base-pair rungs connect them. Strands
  in front are drawn brighter than those behind."""

  default_theme = "ocean"

  thresholds = [
    (0.12, " "), (0.24, "."), (0.36, ":"), (0.48, "-"), (0.60, "="),
    (0.72, "+"), (0.82, "*"), (0.92, "#"), (1.01, "@"),
  ]

  def render(self, width, height, elapsed, phase, options):
    cx = width * 0.5
    amp = width * 0.30
    twist = 0.45 * max(0.4, options.scale)    # turns per row
    t = elapsed * 2.2
    contrast = options.contrast

    grid = [[0.0] * width for _ in range(height)]

    def splat(col, row, value):
      c = int(round(col))
      if 0 <= c < width and 0 <= row < height and grid[row][c] < value:
        grid[row][c] = value

    for row in range(height):
      p = row * twist + t
      s = math.sin(p)
      depthA = math.cos(p)               # +1 front, -1 back
      xA = cx + amp * s
      xB = cx - amp * s
      depthB = -depthA
      brightA = 0.45 + 0.55 * (depthA * 0.5 + 0.5)
      brightB = 0.45 + 0.55 * (depthB * 0.5 + 0.5)
      splat(xA, row, brightA)
      splat(xA + 1, row, brightA * 0.7)
      splat(xB, row, brightB)
      splat(xB + 1, row, brightB * 0.7)
      # base-pair rungs when the strands are spread apart (helix facing us)
      if abs(s) > 0.30 and row % 1 == 0 and (row % 2 == 0):
        x0, x1 = sorted((xA, xB))
        rung = 0.25 + 0.30 * (abs(depthA) < 0.6)
        steps = int(x1 - x0)
        for i in range(1, max(1, steps)):
          splat(x0 + i, row, max(rung, 0.30))
    return render_field(width, height, grid, options, self)
