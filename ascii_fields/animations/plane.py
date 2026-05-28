import math

from ..core import Animation, clamp, render_block_field, render_field, shade


WAVES = (
  (0.26, 1, 2, 1, 0.00),
  (0.21, 2, -1, -1, 0.23),
  (0.18, 3, 1, 2, 0.41),
  (0.15, -2, 3, -2, 0.67),
  (0.13, 4, -3, 3, 0.11),
  (0.10, 5, 2, -3, 0.52),
  (0.08, -4, -5, 4, 0.31),
  (0.06, 7, -2, -4, 0.79),
  (0.05, 6, 5, 5, 0.18),
)

RAMPS = {
  "clean": " .'`^\",:;Il!i~+_-?][}{1)(|\\/tfjrxnuvczXYUJCLQ0OZmwqpdbkhao*#MW&8%B@$",
  "soft": " .'`^\",:-_~+ito+xzMW#@$8",
  "dense": " .,:;irsXA253hMHGS#9B&@",
}


def _ramp_thresholds(ramp):
  n = len(ramp)
  return [(min(1.01, (i + 1) / n), ramp[i]) for i in range(n)]


class WavePlaneAnimation(Animation):
  """The original cyclic wave plane. Now char-rendered through the shared
  pipeline so themes apply and the full ramp can be used; pass --blocks to fall
  back to the smooth coloured-block look."""

  def render(self, width, height, elapsed, phase, options):
    grid = self._levels(width, height, phase, options)
    if options.blocks:
      return render_block_field(width, height, grid, options)
    ramp = RAMPS.get(options.charset, RAMPS["clean"])
    self.thresholds = _ramp_thresholds(ramp)
    return render_field(width, height, grid, options, self)

  def _levels(self, width, height, phase, options):
    grid = []
    for row in range(height):
      v = row / height
      line = []
      for col in range(width):
        u = col / width
        su, sv = self.sample_uv(u, v, phase, options.scroll)
        line.append(clamp(shade(self.wave_height(su, sv, phase, options.scale), options.contrast)))
      grid.append(line)
    return grid

  def sample_uv(self, u, v, phase, scroll):
    if not scroll:
      return u, v
    return (
      (u + 0.08 * math.cos(2.0 * math.pi * phase)) % 1.0,
      (v + 0.08 * math.sin(2.0 * math.pi * phase)) % 1.0,
    )

  def wave_height(self, u, v, phase, scale):
    x = 2.0 * math.pi * u
    y = 2.0 * math.pi * v
    t = 2.0 * math.pi * phase
    density = max(1, round(scale))
    z = 0.0
    for amplitude, wave_x, wave_y, time_speed, offset in WAVES:
      z += amplitude * math.sin(
        (wave_x * density) * x
        + (wave_y * density) * y
        + time_speed * t
        + offset * 2.0 * math.pi
      )
    return z
