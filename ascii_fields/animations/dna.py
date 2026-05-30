import math

from ..core import Animation, render_glyph_field


class DNAAnimation(Animation):
  """A rotating DNA double helix. The two sugar-phosphate backbones spiral
  around each other a half-turn apart and base-pair rungs connect them. Each
  strand is drawn with /, \\ or | depending on which way it is heading at that
  row, so the twist reads as actual twist; the strand currently in front is
  drawn brighter than the one in back."""

  default_theme = "ocean"

  thresholds = [
    (0.12, " "), (0.24, "."), (0.36, ":"), (0.48, "-"), (0.60, "="),
    (0.72, "+"), (0.82, "*"), (0.92, "#"), (1.01, "@"),
  ]

  def render(self, width, height, elapsed, phase, options):
    cx = width * 0.5
    amp = max(3.0, width * 0.20)
    # Twist tuned so ~1.5 turns fit a typical screen height; scale lets users
    # crank it tighter.
    twist = 0.36 * max(0.4, options.scale)
    t = elapsed * 1.8

    grid = [[0.0] * width for _ in range(height)]
    glyphs = [[" "] * width for _ in range(height)]

    def splat(col, row, value, glyph):
      c = int(round(col))
      if 0 <= c < width and 0 <= row < height and grid[row][c] < value:
        grid[row][c] = value
        glyphs[row][c] = glyph

    # -- rungs first so the strands overdraw them at crossings ---------------
    for row in range(height):
      p = row * twist + t
      s = math.sin(p)
      if abs(s) > 0.18:
        xA = cx + amp * s
        xB = cx - amp * s
        x0, x1 = sorted((xA, xB))
        spread = abs(s)
        rung_b = 0.30 + 0.20 * spread
        steps = max(2, int(x1 - x0))
        for i in range(1, steps):
          # alternate thick/thin so a rung looks like a ladder rung, not a line
          glyph = "=" if i % 2 == 0 else "-"
          splat(x0 + i, row, rung_b, glyph)

    # -- strands: A on +amp*sin(p), B on the opposite side --------------------
    for sign in (+1, -1):
      prev_x = None
      for row in range(height):
        p = row * twist + t
        s = math.sin(p)
        c = math.cos(p)
        x = cx + sign * amp * s
        depth = c * sign                              # +1 = front, -1 = back
        slope = sign * amp * c * twist                # dx/drow
        bright = 0.55 + 0.45 * (depth * 0.5 + 0.5)    # 0.55 (back) .. 1.0 (front)
        if abs(slope) < 0.35:
          glyph = "|"                                 # near-vertical at extremes
        elif slope > 0:
          glyph = "\\"                                # heading right going down
        else:
          glyph = "/"                                 # heading left going down
        splat(x, row, bright, glyph)
        splat(x - 1, row, bright * 0.55, glyph)
        splat(x + 1, row, bright * 0.55, glyph)
        # If the strand jumped sideways more than a cell since the previous
        # row, fill the gap so the helix reads as a continuous line.
        if prev_x is not None and abs(x - prev_x) > 1.4:
          step = -1 if x < prev_x else 1
          for fill in range(int(prev_x) + step, int(x), step):
            splat(fill, row, bright * 0.45, glyph)
        prev_x = x

    return render_glyph_field(width, height, grid, glyphs, options, self)
