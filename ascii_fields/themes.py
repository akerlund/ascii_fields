"""Colour themes for truecolor (24-bit) terminals.

A theme is a list of gradient stops ``(position, (r, g, b))`` sorted by
position in [0, 1]. :func:`palette` returns a memoised lookup that maps a
brightness level to an interpolated colour. The default "mono" theme is handled
directly in :mod:`core` as a 256-colour grayscale ramp and is included here only
so ``--theme mono --truecolor`` style requests still resolve.
"""

GRADIENTS = {
  "mono": [(0.0, (24, 24, 24)), (1.0, (255, 255, 255))],
  "fire": [
    (0.0, (0, 0, 0)), (0.30, (90, 12, 4)), (0.55, (200, 55, 10)),
    (0.78, (255, 150, 25)), (0.92, (255, 225, 110)), (1.0, (255, 255, 230)),
  ],
  "ice": [
    (0.0, (0, 0, 0)), (0.30, (8, 22, 60)), (0.55, (20, 80, 150)),
    (0.78, (70, 170, 220)), (1.0, (225, 250, 255)),
  ],
  "nebula": [
    (0.0, (0, 0, 0)), (0.28, (40, 8, 70)), (0.52, (120, 25, 140)),
    (0.74, (210, 60, 160)), (0.90, (250, 150, 200)), (1.0, (255, 240, 250)),
  ],
  "aurora": [
    (0.0, (0, 0, 0)), (0.30, (4, 40, 35)), (0.52, (20, 140, 90)),
    (0.72, (70, 220, 150)), (0.88, (160, 250, 210)), (1.0, (235, 255, 245)),
  ],
  "amber": [
    (0.0, (0, 0, 0)), (0.35, (60, 35, 5)), (0.60, (160, 95, 15)),
    (0.82, (235, 170, 40)), (1.0, (255, 240, 190)),
  ],
  "plasma": [
    (0.0, (12, 8, 70)), (0.30, (90, 10, 140)), (0.55, (190, 40, 120)),
    (0.78, (240, 110, 60)), (0.92, (250, 200, 70)), (1.0, (250, 255, 170)),
  ],
  "ocean": [
    (0.0, (0, 0, 0)), (0.30, (4, 30, 55)), (0.55, (10, 90, 110)),
    (0.78, (40, 170, 175)), (0.92, (160, 230, 225)), (1.0, (240, 255, 255)),
  ],
  "spectrum": [
    (0.0, (10, 0, 30)), (0.20, (60, 0, 160)), (0.40, (0, 130, 220)),
    (0.55, (0, 200, 120)), (0.70, (220, 220, 0)), (0.85, (240, 110, 20)),
    (1.0, (250, 60, 60)),
  ],
}

THEME_NAMES = tuple(GRADIENTS.keys())
# Order used by the HUD's `t` key to cycle through themes at runtime.
THEME_CYCLE = ("auto", "scene", *THEME_NAMES)

_CACHE = {}


def _build(stops, steps=256):
  table = []
  for i in range(steps):
    pos = i / (steps - 1)
    lo = stops[0]
    hi = stops[-1]
    for a, b in zip(stops, stops[1:]):
      if a[0] <= pos <= b[0]:
        lo, hi = a, b
        break
    span = hi[0] - lo[0]
    f = 0.0 if span <= 0 else (pos - lo[0]) / span
    color = tuple(round(lo[1][c] + (hi[1][c] - lo[1][c]) * f) for c in range(3))
    table.append(color)
  return table


def palette(name):
  """Return a callable ``level -> (r, g, b)`` for the given theme name."""
  table = _CACHE.get(name)
  if table is None:
    table = _build(GRADIENTS.get(name, GRADIENTS["mono"]))
    _CACHE[name] = table
  last = len(table) - 1

  def lookup(level, _table=table, _last=last):
    idx = int(level * _last + 0.5)
    if idx < 0:
      idx = 0
    elif idx > _last:
      idx = _last
    return _table[idx]

  return lookup
