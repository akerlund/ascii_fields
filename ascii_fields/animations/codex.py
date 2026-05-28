import math

from ..core import Animation, clamp, render_field


RAMP = "  ..::--==++**##%%"
_TOP = len(RAMP) - 1


class FlowerSphereAnimation(Animation):
  thresholds = [(min(1.01, (i + 1) / len(RAMP)), RAMP[i]) for i in range(len(RAMP))]

  def render(self, width, height, elapsed, phase, options):
    radius = max(1.0, min(width, height * 2) * 0.5)
    grid = []
    for row in range(height):
      py = ((row - (height - 1) * 0.5) * 2.0) / radius
      line = []
      for col in range(width):
        px = (col - (width - 1) * 0.5) / radius
        r = math.hypot(px, py)
        if r >= 1.0:
          line.append(0.0)
          continue
        u = (math.atan2(py, px) / (2.0 * math.pi) + 0.5
             + 0.025 * math.sin(2.0 * math.pi * phase)) % 1.0
        v = (math.asin(clamp(py, -1.0, 1.0)) / math.pi) + 0.5
        texture = self.texture(u, v, phase, options.scale)
        edge_fade = max(0.0, 1.0 - r ** 2.8)
        center_dip = 1.0 - 0.18 * math.exp(-7.0 * r * r)
        line.append(clamp(texture * edge_fade * center_dip * options.contrast))
      grid.append(line)
    return render_field(width, height, grid, options, self)

  def texture(self, u, v, phase, scale):
    x = 2.0 * math.pi * u
    y = 2.0 * math.pi * v
    t = 2.0 * math.pi * phase
    density = max(1, round(scale))
    vertical = abs(math.sin((5 * density) * x + 0.85 * math.sin(3.0 * y + t)))
    diagonal = abs(math.sin((3 * density) * x - (2 * density) * y - t))
    fine = abs(math.sin((11 * density) * x + (4 * density) * y + 2.0 * t))
    interference = abs(math.sin((2 * density) * x + math.sin(5.0 * y - t)))
    return 0.38 * vertical + 0.25 * diagonal + 0.22 * fine + 0.15 * interference
