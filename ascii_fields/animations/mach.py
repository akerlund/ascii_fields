import math

from ..core import Animation, clamp, render_field


CYCLE = 11.0


class MachAnimation(Animation):
  """An object crossing the screen while emitting sound. Each pulse spreads as
  a circle at the speed of sound; as the object accelerates from subsonic to
  supersonic the circles bunch up and then pile into a Mach cone -- a sonic
  boom. The Mach number is shown growing past 1 across each pass."""

  thresholds = [
    (0.12, " "), (0.22, "."), (0.34, ":"), (0.46, "-"), (0.58, "="),
    (0.70, "+"), (0.82, "*"), (0.92, "#"), (1.01, "@"),
  ]

  WAVE_SPEED = 0.16          # normalised units per second
  EMIT_DT = 0.10

  def __init__(self):
    self._fronts = []        # (emit_x, emit_y, emit_time)
    self._cycle_start = 0.0
    self._next_emit = 0.0
    self._last = 0.0

  def _mach(self, p):
    # accelerate from 0.6 -> ~2.4 across the pass
    return 0.6 + 1.9 * p

  def render(self, width, height, elapsed, phase, options):
    if elapsed < self._last or elapsed - self._cycle_start > CYCLE:
      self._cycle_start = elapsed
      self._fronts = []
      self._next_emit = elapsed
    self._last = elapsed
    local = elapsed - self._cycle_start

    ax = width / max(1, height * 2.0)
    c = self.WAVE_SPEED
    sy = 0.0
    # integrate source x by sampling its speed profile (closed enough)
    def source_x(t):
      p = t / CYCLE
      return -1.4 * ax + (self._mach(p) * c) * t

    while elapsed >= self._next_emit:
      tl = self._next_emit - self._cycle_start
      self._fronts.append((source_x(tl), sy, tl))
      self._next_emit += self.EMIT_DT
    self._fronts = [f for f in self._fronts if local - f[2] < 14.0]

    sx = source_x(local)
    contrast = options.contrast
    grid = []
    for row in range(height):
      py = (row / max(1, height - 1) - 0.5) * 2.0
      line = []
      for col in range(width):
        px = (col / max(1, width - 1) - 0.5) * 2.0 * ax
        v = 0.0
        for (ex, ey, et) in self._fronts:
          radius = c * (local - et)
          dist = math.hypot(px - ex, py - ey)
          v += math.exp(-((dist - radius) ** 2) / 0.0007)
        # the moving object
        v += 0.9 * math.exp(-((px - sx) ** 2 + py * py) / 0.0009)
        line.append(clamp(v * 0.9 * contrast))
      grid.append(line)
    return render_field(width, height, grid, options, self)
