import math

from ..core import Animation, clamp, render_glyph_field


# Each diagram is a list of elements:
#   ("fermion"|"photon"|"gluon", (x0, y0), (x1, y1))
#   ("vertex", (x, y))
#   ("label", (x, y), "text")
# Coordinates are normalised to [0, 1].
DIAGRAMS = [
  # e+ e-  ->  (virtual photon)  ->  mu+ mu-
  [
    ("label", (0.02, 0.18), "e-"),
    ("label", (0.02, 0.82), "e+"),
    ("fermion", (0.06, 0.20), (0.40, 0.50)),
    ("fermion", (0.06, 0.80), (0.40, 0.50)),
    ("vertex", (0.40, 0.50)),
    ("photon", (0.40, 0.50), (0.60, 0.50)),
    ("vertex", (0.60, 0.50)),
    ("fermion", (0.60, 0.50), (0.94, 0.20)),
    ("fermion", (0.60, 0.50), (0.94, 0.80)),
    ("label", (0.95, 0.18), "u-"),
    ("label", (0.95, 0.82), "u+"),
  ],
  # Compton scattering: e- absorbs a photon and re-emits one
  [
    ("label", (0.02, 0.30), "y"),
    ("label", (0.02, 0.86), "e-"),
    ("photon", (0.06, 0.28), (0.34, 0.62)),
    ("fermion", (0.06, 0.86), (0.34, 0.62)),
    ("vertex", (0.34, 0.62)),
    ("fermion", (0.34, 0.62), (0.66, 0.62)),
    ("vertex", (0.66, 0.62)),
    ("photon", (0.66, 0.62), (0.94, 0.28)),
    ("fermion", (0.66, 0.62), (0.94, 0.86)),
    ("label", (0.95, 0.30), "y"),
    ("label", (0.95, 0.86), "e-"),
  ],
  # quark-quark scattering via gluon exchange (t-channel)
  [
    ("label", (0.02, 0.18), "q"),
    ("label", (0.02, 0.82), "q"),
    ("fermion", (0.06, 0.20), (0.42, 0.32)),
    ("fermion", (0.06, 0.80), (0.42, 0.68)),
    ("vertex", (0.42, 0.32)),
    ("vertex", (0.42, 0.68)),
    ("gluon", (0.42, 0.32), (0.42, 0.68)),
    ("fermion", (0.42, 0.32), (0.94, 0.20)),
    ("fermion", (0.42, 0.68), (0.94, 0.80)),
    ("label", (0.95, 0.18), "q"),
    ("label", (0.95, 0.82), "q"),
  ],
]

DIAGRAM_SECONDS = 4.5
TRAVEL = 2.0


def _line_char(angle):
  a = (math.degrees(angle) + 360) % 180
  if a < 22.5 or a >= 157.5:
    return "-"
  if a < 67.5:
    return "\\"
  if a < 112.5:
    return "|"
  return "/"


def _arrow(angle):
  deg = (math.degrees(angle) + 360) % 360
  if deg < 45 or deg >= 315:
    return ">"
  if deg < 135:
    return "v"
  if deg < 225:
    return "<"
  return "^"


class FeynmanAnimation(Animation):
  """Animated Feynman diagrams. Straight arrowed lines are fermions, wavy lines
  are photons, coiled lines are gluons, and dots are interaction vertices. A
  pulse of brightness flows through each diagram, which cycles every few
  seconds."""

  gray_lo = 234
  gray_hi = 255

  def render(self, width, height, elapsed, phase, options):
    diagram = DIAGRAMS[int(elapsed / DIAGRAM_SECONDS) % len(DIAGRAMS)]
    tpos = (elapsed % TRAVEL) / TRAVEL
    grid = [[0.0] * width for _ in range(height)]
    glyphs = [[" "] * width for _ in range(height)]

    def put(x, y, value, ch):
      c, r = int(round(x)), int(round(y))
      if 0 <= c < width and 0 <= r < height and value >= grid[r][c]:
        grid[r][c] = value
        glyphs[r][c] = ch

    for element in diagram:
      kind = element[0]
      if kind == "vertex":
        x, y = element[1]
        put(x * (width - 1), y * (height - 1), 1.0, "@")
        continue
      if kind == "label":
        (x, y), text = element[1], element[2]
        col0 = int(x * (width - 1))
        for k, ch in enumerate(text):
          put(col0 + k, y * (height - 1), 0.8, ch)
        continue
      (x0, y0), (x1, y1) = element[1], element[2]
      cx0, cy0 = x0 * (width - 1), y0 * (height - 1)
      cx1, cy1 = x1 * (width - 1), y1 * (height - 1)
      dx, dy = cx1 - cx0, cy1 - cy0
      length = math.hypot(dx, dy) or 1.0
      steps = max(2, int(length))
      perp_x, perp_y = -dy / length, dx / length
      angle = math.atan2(dy, dx)
      base_char = _line_char(angle)
      for s in range(steps + 1):
        f = s / steps
        x = cx0 + dx * f
        y = cy0 + dy * f
        ch = base_char
        if kind == "photon":
          wig = 1.3 * math.sin(s * 0.7)
          x += perp_x * wig
          y += perp_y * wig
          ch = "~"
        elif kind == "gluon":
          wig = 1.6 * math.sin(s * 1.0)
          x += perp_x * wig
          y += perp_y * wig
          ch = "o"
        pulse = math.exp(-((f - tpos) ** 2) / 0.01)
        put(x, y, clamp(0.32 + 0.75 * pulse), ch)
      if kind == "fermion":
        put(cx0 + dx * 0.55, cy0 + dy * 0.55, 1.0, _arrow(angle))
    return render_glyph_field(width, height, grid, glyphs, options, self)
