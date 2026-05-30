import math

from ..core import Animation, clamp, render_glyph_field


NODES = (
  (0.04, 0.10, "E"), (0.18, 0.06, "o"), (0.38, 0.08, "S"), (0.62, 0.06, "o"),
  (0.82, 0.10, "S"), (0.96, 0.18, "E"),
  (0.08, 0.36, "o"), (0.24, 0.30, "R"), (0.46, 0.33, "o"), (0.66, 0.28, "R"),
  (0.88, 0.38, "o"), (0.98, 0.52, "E"),
  (0.03, 0.66, "E"), (0.20, 0.70, "o"), (0.42, 0.63, "R"), (0.60, 0.70, "o"),
  (0.78, 0.64, "R"), (0.94, 0.78, "o"),
  (0.14, 0.92, "E"), (0.34, 0.86, "o"), (0.55, 0.92, "S"), (0.76, 0.88, "o"),
  (0.96, 0.94, "E"),
)

EDGES = (
  (0, 1), (1, 2), (2, 3), (3, 4), (4, 5),
  (0, 6), (1, 7), (2, 7), (2, 8), (3, 8), (4, 9), (5, 10), (5, 11),
  (6, 7), (7, 8), (8, 9), (9, 10), (10, 11),
  (6, 12), (7, 13), (8, 14), (9, 15), (10, 16), (11, 17),
  (12, 13), (13, 14), (14, 15), (15, 16), (16, 17),
  (12, 18), (13, 19), (14, 19), (14, 20), (15, 20), (16, 21), (17, 22),
  (18, 19), (19, 20), (20, 21), (21, 22),
  (7, 14), (9, 14), (9, 16), (14, 20), (4, 16),
)

PACKETS = (
  ("@", 1.00, 0.085),
  ("*", 0.86, 0.115),
  ("+", 0.72, 0.145),
)


def _line_char(dx, dy):
  if abs(dx) > abs(dy) * 1.8:
    return "-"
  if abs(dy) > abs(dx) * 1.8:
    return "|"
  return "\\" if dx * dy > 0 else "/"


class NetworkAnimation(Animation):
  """A network topology spanning the screen, with routers, switches, endpoints
  and multiple packet classes moving over the links."""

  default_theme = "copper"

  def render(self, width, height, elapsed, phase, options):
    grid = [[0.0] * width for _ in range(height)]
    glyphs = [[" "] * width for _ in range(height)]
    if width <= 0 or height <= 0:
      return ""

    # Nodes are stationary; the previous wobble was sub-pixel and invisible.
    nodes = list(NODES)

    def put(col, row, value, glyph):
      if 0 <= col < width and 0 <= row < height and value >= grid[row][col]:
        grid[row][col] = value
        glyphs[row][col] = glyph

    def point(node):
      x, y, _ = nodes[node]
      return round(x * (width - 1)), round(y * (height - 1))

    for edge_idx, (a, b) in enumerate(EDGES):
      c0, r0 = point(a)
      c1, r1 = point(b)
      dx, dy = c1 - c0, r1 - r0
      steps = max(1, int(math.hypot(dx, dy)))
      char = _line_char(dx, dy)
      load = 0.17 + 0.06 * math.sin(elapsed * 0.8 + edge_idx)
      for step in range(steps + 1):
        f = step / steps
        put(round(c0 + dx * f), round(r0 + dy * f), load, char)

      # Offset packets perpendicular to the edge so multiple classes on the
      # same edge don't sit on top of each other and turn into mush.
      length = max(1.0, math.hypot(dx, dy))
      perp_x = -dy / length
      perp_y = dx / length
      for packet_idx, (glyph, brightness, speed) in enumerate(PACKETS):
        if (edge_idx + packet_idx) % 3 == 2:
          continue
        pulse = (elapsed * (speed + 0.006 * (edge_idx % 7))
                 + edge_idx * 0.091 + packet_idx * 0.27) % 1.0
        if (edge_idx + packet_idx) % 4 == 0:
          pulse = 1.0 - pulse
        lane = (packet_idx - 1) * 0.6   # -0.6, 0, +0.6 cells off the edge
        ox = perp_x * lane
        oy = perp_y * lane
        for tail in range(4):
          f = pulse - tail * 0.030
          if f < 0.0 or f > 1.0:
            continue
          value = brightness - tail * 0.13
          put(round(c0 + dx * f + ox), round(r0 + dy * f + oy),
              value, glyph if tail == 0 else ".")

    for idx, (_, _, kind) in enumerate(nodes):
      col, row = point(idx)
      beat = 0.70 + 0.30 * math.sin(elapsed * 1.5 + idx * 0.9)
      if kind == "R":
        put(col, row, 1.0, "#")
        for dc, dr in ((1, 0), (-1, 0), (0, 1), (0, -1)):
          put(col + dc, row + dr, 0.45, "+")
      elif kind == "S":
        put(col, row, 0.90 + 0.10 * beat, "X")
        put(col - 1, row, 0.40, "=")
        put(col + 1, row, 0.40, "=")
      elif kind == "E":
        # Single distinctive glyph for endpoints rather than alternating brackets.
        put(col, row, 0.78 + 0.20 * beat, "O")
      else:
        put(col, row, 0.66 + 0.25 * beat, "o")
        put(col, row - 1, 0.24, ".")

    contrast = options.contrast
    return render_glyph_field(
      width, height,
      [[clamp(value * contrast) for value in row] for row in grid],
      glyphs, options, self
    )
