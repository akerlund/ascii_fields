import math

from ..core import Animation, clamp, render_field, smoothstep


CYCLE = 8.5


class MachAnimation(Animation):
  """An object crossing the screen while emitting sound. Each pulse spreads as
  a circle at the speed of sound; as the object accelerates from subsonic to
  supersonic the circles bunch up and then pile into a Mach cone -- a sonic
  boom. The Mach number is shown growing past 1 across each pass."""

  thresholds = [
    (0.12, " "), (0.22, "."), (0.34, ":"), (0.46, "-"), (0.58, "="),
    (0.70, "+"), (0.82, "*"), (0.92, "#"), (1.01, "@"),
  ]

  WAVE_SPEED = 0.18
  LANES = (-0.28, 0.22)

  def _mach(self, p, seed):
    top = 1.85 + 0.45 * seed
    return 0.70 + top * smoothstep(0.04, 0.92, p)

  def render(self, width, height, elapsed, phase, options):
    ax = width / max(1, height * 2.0)
    contrast = options.contrast
    grid = []
    for row in range(height):
      py = (row / max(1, height - 1) - 0.5) * 2.0
      line = []
      for col in range(width):
        px = (col / max(1, width - 1) - 0.5) * 2.0 * ax
        v = 0.0
        for idx, lane_y in enumerate(self.LANES):
          local = (elapsed / CYCLE + idx * 0.50) % 1.0
          seed = idx + 1
          mach = self._mach(local, seed)
          sx = (-1.45 + 3.10 * local) * ax
          sy = lane_y + 0.035 * math.sin(elapsed * 0.55 + idx * 2.4)
          cone_angle = math.asin(1.0 / mach) if mach > 1.0 else math.pi * 0.5
          cone_slope = math.tan(cone_angle)

          dist = math.hypot(px - sx, py - sy)
          rings = math.sin((dist - elapsed * self.WAVE_SPEED * (1.0 + idx * 0.1)) * 52.0)
          v += max(0.0, rings) * 0.12 * math.exp(-dist * 1.6)

          if mach > 1.0 and px < sx:
            behind = sx - px
            edge = abs(py - sy) - behind * cone_slope
            cone = math.exp(-(edge * edge) / 0.0012) * math.exp(-behind * 0.28)
            interior = smoothstep(0.10, 0.0, edge) * 0.14 * math.exp(-behind * 0.22)
            v += cone * (1.04 + idx * 0.15) + interior

          v += 0.86 * math.exp(-((px - sx) ** 2 + (py - sy) ** 2) / 0.0008)
        line.append(clamp(v * contrast))
      grid.append(line)
    return render_field(width, height, grid, options, self)
