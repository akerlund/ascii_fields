from ..core import Animation, clamp, render_field


SIGMA = 10.0
RHO = 28.0
BETA = 8.0 / 3.0
DT = 0.012
STEPS_PER_FRAME = 50
TRAIL_DECAY = 0.94


class LorenzAnimation(Animation):
  """The Lorenz strange attractor. Integrating x' = sigma(y-x), y' = x(rho-z)-y,
  z' = xy - beta z traces a chaotic butterfly: never the same trajectory twice
  but always the same shape. The current point glows; the trail fades behind."""

  thresholds = [
    (0.05, " "), (0.18, "."), (0.32, ":"), (0.46, "-"), (0.60, "="),
    (0.72, "+"), (0.84, "*"), (0.93, "#"), (1.01, "@"),
  ]

  def __init__(self):
    self._w = 0
    self._h = 0
    self._grid = None
    self._x = 0.1
    self._y = 0.0
    self._z = 0.0
    self._last = 0.0

  def _seed(self, w, h):
    self._w, self._h = w, h
    self._grid = [0.0] * (w * h)
    self._x, self._y, self._z = 0.1, 0.0, 0.0

  def render(self, width, height, elapsed, phase, options):
    if self._grid is None or width != self._w or height != self._h or elapsed < self._last:
      self._seed(width, height)
    self._last = elapsed

    grid = self._grid
    decay = TRAIL_DECAY ** max(0.5, options.scale)
    for i in range(len(grid)):
      grid[i] *= decay

    x, y, z = self._x, self._y, self._z
    fx = width * 0.5 / 30.0          # horizontal: x in [-22, 22]
    fy = height * 0.5 / 30.0         # vertical: z centered around 25
    cx, cy = width * 0.5, height * 0.5
    steps = int(STEPS_PER_FRAME * max(0.4, options.scale))
    for _ in range(steps):
      dx = SIGMA * (y - x)
      dy = x * (RHO - z) - y
      dz = x * y - BETA * z
      x += dx * DT
      y += dy * DT
      z += dz * DT
      ci = int(cx + x * fx)
      ri = int(cy + (z - 25.0) * fy)
      if 0 <= ci < width and 0 <= ri < height:
        idx = ri * width + ci
        if grid[idx] < 1.0:
          grid[idx] = min(1.0, grid[idx] + 0.55)
    self._x, self._y, self._z = x, y, z

    contrast = options.contrast
    out = []
    for r in range(height):
      base = r * width
      out.append([clamp(grid[base + c] * contrast) for c in range(width)])
    return render_field(width, height, out, options, self)
