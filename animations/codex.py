import math

from core import Animation, BLACK_BG, RESET, gray_fg


class FlowerSphereAnimation(Animation):
  uses_elapsed_time = False

  def render(self, width, height, elapsed, phase, options):
    ramp = "  ..::--==++**##%%"
    radius = max(1.0, min(width, height * 2) * 0.5)
    rows = []
    for row in range(height):
      line = [BLACK_BG]
      last_color = None
      for col in range(width):
        px = (col - (width - 1) * 0.5) / radius
        py = ((row - (height - 1) * 0.5) * 2.0) / radius
        r = math.hypot(px, py)
        if r >= 1.0:
          if last_color != 233:
            line.append("\x1b[38;5;233m")
            last_color = 233
          line.append(" ")
          continue

        u = (math.atan2(py, px) / (2.0 * math.pi) + 0.5 + 0.025 * math.sin(2.0 * math.pi * phase)) % 1.0
        v = (math.asin(max(-1.0, min(1.0, py))) / math.pi) + 0.5
        texture = self.texture(u, v, phase, options.scale)
        edge_fade = max(0.0, 1.0 - r ** 2.8)
        center_dip = 1.0 - 0.18 * math.exp(-7.0 * r * r)
        level = max(0.0, min(1.0, texture * edge_fade * center_dip * options.contrast))
        char = ramp[round(level * (len(ramp) - 1))]
        color = gray_fg(level)
        if last_color != color:
          line.append(f"\x1b[38;5;{color}m")
          last_color = color
        line.append(char)
      line.append(RESET)
      rows.append("".join(line))
    return "\n".join(rows)

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
