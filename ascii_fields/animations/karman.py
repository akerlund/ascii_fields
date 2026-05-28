import math

from ..core import Animation, clamp, render_field


class KarmanAnimation(Animation):
  """A Karman vortex street. Fluid streaming past a circular obstacle sheds
  alternating swirls that drift downstream and dissipate. The vortices appear
  here as glowing blobs spinning off the obstacle's wake in alternating rows."""

  thresholds = [
    (0.10, " "), (0.22, "."), (0.34, ":"), (0.46, "-"), (0.58, "="),
    (0.70, "+"), (0.82, "*"), (0.92, "#"), (1.01, "@"),
  ]

  SHED_DT = 0.95
  FLOW_SPEED = 0.55
  OBSTACLE_X = -0.55
  OBSTACLE_R = 0.10
  OFFSET = 0.16

  def __init__(self):
    self._vortices = []      # (offset_y_sign, t0)
    self._next_shed = 0.0
    self._last = 0.0
    self._even = False

  def render(self, width, height, elapsed, phase, options):
    if elapsed < self._last:
      self._vortices = []
      self._next_shed = 0.0
      self._even = False
    self._last = elapsed
    while elapsed >= self._next_shed:
      sign = +1 if self._even else -1
      self._vortices.append((sign, self._next_shed))
      self._even = not self._even
      self._next_shed += self.SHED_DT
    self._vortices = [(s, t0) for (s, t0) in self._vortices if elapsed - t0 < 10.0]

    ax = width / max(1, height * 2.0)
    contrast = options.contrast
    grid = []
    for row in range(height):
      py = (row / max(1, height - 1) - 0.5) * 2.0
      line = []
      for col in range(width):
        px = (col / max(1, width - 1) - 0.5) * 2.0 * ax
        # the obstacle: dark disk
        dx_obs = px - self.OBSTACLE_X
        d_obs = math.hypot(dx_obs, py)
        if d_obs < self.OBSTACLE_R:
          line.append(0.0)
          continue
        # gentle background flow brightness, brighter past the obstacle
        bg = 0.10 + 0.04 * math.sin(8.0 * py + 0.6 * elapsed)
        # sum each vortex's contribution
        accum = 0.0
        for sign, t0 in self._vortices:
          age = elapsed - t0
          vx = self.OBSTACLE_X + self.OBSTACLE_R + self.FLOW_SPEED * age
          vy = sign * self.OFFSET + sign * 0.04 * math.sin(1.6 * age)
          r2 = (px - vx) ** 2 + (py - vy) ** 2
          sigma2 = 0.0030 + 0.0035 * age
          accum += sign * math.exp(-r2 / (2.0 * sigma2)) * math.exp(-age * 0.16) * 0.9
        line.append(clamp((bg + abs(accum) * 1.4) * contrast))
      grid.append(line)
    return render_field(width, height, grid, options, self)
