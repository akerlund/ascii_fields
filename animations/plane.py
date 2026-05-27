import math

from core import Animation, RESET, shade


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

ASCII_RAMPS = {
  "clean": " .'`^\",:;Il!i~+_-?][}{1)(|\\/tfjrxnuvczXYUJCLQ0OZmwqpdbkhao*#MW&8%B@$",
  "soft": " .'`^\",:-_~+ito+xzMW#@$8",
  "dense": " .,:;irsXA253hMHGS#9B&@",
}


class WavePlaneAnimation(Animation):
  uses_elapsed_time = False

  def render(self, width, height, elapsed, phase, options):
    if options.ascii_mode:
      return self.render_ascii(width, height, phase, options)
    return self.render_blocks(width, height, phase, options)

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

  def render_blocks(self, width, height, phase, options):
    rows = []
    for row in range(height):
      v = row / height
      line = []
      last_gray = None
      for col in range(width):
        u = col / width
        sample_u, sample_v = self.sample_uv(u, v, phase, options.scroll)
        level = shade(self.wave_height(sample_u, sample_v, phase, options.scale), options.contrast)
        gray = 232 + round(level * 23)
        if gray != last_gray:
          line.append(f"\x1b[48;5;{gray}m")
          last_gray = gray
        line.append(" ")
      line.append(RESET)
      rows.append("".join(line))
    return "\n".join(rows)

  def render_ascii(self, width, height, phase, options):
    ramp = ASCII_RAMPS[options.charset]
    rows = []
    for row in range(height):
      v = row / height
      chars = []
      for col in range(width):
        u = col / width
        sample_u, sample_v = self.sample_uv(u, v, phase, options.scroll)
        level = shade(self.wave_height(sample_u, sample_v, phase, options.scale), options.contrast)
        chars.append(ramp[round(level * (len(ramp) - 1))])
      rows.append("".join(chars))
    return "\n".join(rows)
