import random

from ..core import Animation, render_field


STEP_DT = 0.11          # seconds per generation
RESEED_GENERATIONS = 900
INJECT_EVERY = 48


class LifeAnimation(Animation):
  """Conway's Game of Life on a toroidal grid. Cells leave a fading trail so
  gliders and oscillators draw glowing streaks. Reseeds when the field dies or
  stagnates."""

  thresholds = [
    (0.08, " "), (0.20, "."), (0.34, ":"), (0.48, "-"), (0.62, "="),
    (0.74, "+"), (0.85, "*"), (0.93, "#"), (1.01, "@"),
  ]

  def __init__(self):
    self._w = 0
    self._h = 0
    self._cells = None       # set of (x, y) alive
    self._age = None         # flat list: frames since last alive (trail)
    self._gen = 0
    self._last_elapsed = 0.0
    self._rng = random.Random()
    self._stable = 0
    self._last_pop = -1
    self._signature = None
    self._signature_stable = 0

  def _seed(self, w, h):
    self._w, self._h = w, h
    self._cells = set()
    fill = 0.12
    for y in range(1, max(1, h - 1)):
      for x in range(1, max(1, w - 1)):
        if self._rng.random() < fill:
          self._cells.add((x, y))
    for _ in range(max(4, (w * h) // 180)):
      self._add_pattern(self._rng.randrange(w), self._rng.randrange(h))
    self._age = [999] * (w * h)
    for (x, y) in self._cells:
      self._age[y * w + x] = 0
    self._gen = 0
    self._stable = 0
    self._last_pop = -1
    self._signature = None
    self._signature_stable = 0

  def _add_pattern(self, ox, oy):
    patterns = (
      ((1, 0), (2, 1), (0, 2), (1, 2), (2, 2)),                 # glider
      ((1, 0), (2, 0), (3, 0), (2, 1), (1, 2)),                 # lightweight spark
      ((0, 1), (1, 1), (2, 1), (3, 1), (4, 1)),                 # blinker line
      ((1, 0), (2, 0), (0, 1), (1, 1), (1, 2)),                 # r-pentomino-ish
    )
    pattern = self._rng.choice(patterns)
    flip_x = -1 if self._rng.random() < 0.5 else 1
    flip_y = -1 if self._rng.random() < 0.5 else 1
    for dx, dy in pattern:
      self._cells.add(((ox + dx * flip_x) % self._w, (oy + dy * flip_y) % self._h))

  def _step(self):
    w, h = self._w, self._h
    counts = {}
    for (x, y) in self._cells:
      for dy in (-1, 0, 1):
        ny = (y + dy) % h
        for dx in (-1, 0, 1):
          if dx == 0 and dy == 0:
            continue
          nx = (x + dx) % w
          counts[(nx, ny)] = counts.get((nx, ny), 0) + 1
    new_cells = set()
    for cell, c in counts.items():
      if c == 3 or (c == 2 and cell in self._cells):
        new_cells.add(cell)
    self._cells = new_cells
    age = self._age
    for i in range(len(age)):
      age[i] += 1
    for (x, y) in self._cells:
      age[y * w + x] = 0
    self._gen += 1
    pop = len(self._cells)
    if pop == self._last_pop:
      self._stable += 1
    else:
      self._stable = 0
    self._last_pop = pop
    signature = hash(frozenset(self._cells))
    if signature == self._signature:
      self._signature_stable += 1
    else:
      self._signature_stable = 0
    self._signature = signature
    if self._gen % INJECT_EVERY == 0 and pop < self._w * self._h * 0.22:
      self._add_pattern(self._rng.randrange(self._w), self._rng.randrange(self._h))

  def render(self, width, height, elapsed, phase, options):
    if self._cells is None or width != self._w or height != self._h or elapsed < self._last_elapsed:
      self._seed(width, height)
    self._last_elapsed = elapsed

    target = int(elapsed / STEP_DT)
    guard = 0
    while self._gen < target and guard < 200:
      self._step()
      guard += 1
      if (not self._cells or self._stable > 80 or self._signature_stable > 10
          or self._gen > RESEED_GENERATIONS):
        self._seed(width, height)
        self._gen = target

    age = self._age
    w = self._w
    grid = []
    for y in range(height):
      base = y * w
      line = []
      for x in range(width):
        a = age[base + x]
        if a == 0:
          line.append(1.0)
        elif a < 16:
          line.append(0.72 - 0.04 * a)
        else:
          line.append(0.0)
      grid.append(line)
    return render_field(width, height, grid, options, self)
