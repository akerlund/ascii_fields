import math

from core import Animation, BLACK_BG, RESET, density_char, gray_fg, smoothstep


WAVE_CHAR_THRESHOLDS = [(0.16, " "), (0.24, "."), (0.33, ":"), (0.44, "-"), (0.55, "="), (0.68, "+"), (0.82, "*"), (0.94, "#"), (1.01, "%")]


def wave_char(level):
  return density_char(level, WAVE_CHAR_THRESHOLDS)


class WavesAnimation(Animation):
  def render(self, width, height, elapsed, phase, options):
    density = max(0.45, options.scale)
    t = elapsed
    rows = []
    for row in range(height):
      line = [BLACK_BG]
      last_color = None
      y = row / max(1, height - 1)
      for col in range(width):
        x = col / max(1, width - 1)
        shore = 0.70
        shore += 0.10 * math.sin(2.0 * math.pi * x + 0.20 * math.sin(t * 0.19))
        shore += 0.055 * math.sin(5.4 * math.pi * x - t * 0.11)
        shore += 0.025 * math.sin(12.0 * math.pi * x + 0.13 * t)
        water = 1.0 - smoothstep(shore - 0.045, shore + 0.020, y)
        beach = 1.0 - water
        depth = max(0.0, shore - y)
        shallow = 1.0 - smoothstep(0.02, 0.48, depth)
        shoal_gain = 0.28 + 0.90 * shallow
        sandbar = 0.05 * math.sin(3.0 * math.pi * x - 0.23 * t)
        sandbar += 0.025 * math.sin(9.0 * math.pi * x + 0.31 * t)
        refracted_depth = max(0.0, depth + sandbar * shallow)
        packet_a = 0.62 + 0.38 * math.sin(1.4 * x + 0.31 * t)
        packet_b = 0.64 + 0.36 * math.sin(3.6 * x - 0.17 * t + 1.2)
        approach_a = (18.0 * density) * refracted_depth
        approach_a += 1.8 * math.sin(2.4 * math.pi * x + 0.16 * t)
        approach_b = (11.0 * density) * (refracted_depth + 0.06 * math.sin(5.0 * x - 0.2 * t))
        crest_a = smoothstep(0.78, 0.985, math.sin(approach_a - 2.7 * t))
        crest_b = smoothstep(0.80, 0.990, math.sin(approach_b - 1.6 * t + 1.4))
        incoming = water * shoal_gain * (0.54 * packet_a * crest_a + 0.34 * packet_b * crest_b)
        back_axis = depth + 0.10 * math.sin(4.0 * math.pi * x + 0.2 * t)
        backwash = smoothstep(0.80, 0.99, math.sin((13.0 * density) * back_axis + 1.85 * t))
        backwash *= (0.20 + 0.80 * shallow) * (0.65 + 0.35 * math.sin(2.0 * x - 0.41 * t))
        breaker = math.exp(-42.0 * depth)
        foam_spread = smoothstep(0.18, 0.02, depth)
        shore_foam = water * breaker * (0.35 + 0.65 * max(crest_a, crest_b))
        wash_up = beach * math.exp(-28.0 * max(0.0, y - shore))
        wash_up *= 0.35 + 0.65 * smoothstep(0.25, 0.95, math.sin(8.0 * (y - shore) - 1.35 * t))
        ripples = 0.06 * water * (1.0 - shallow) * abs(math.sin(38.0 * depth + 6.0 * x - 1.1 * t))
        wet_sand = beach * (0.10 + 0.16 * math.exp(-16.0 * max(0.0, y - shore)))
        texture = incoming + 0.42 * backwash + 0.35 * foam_spread * max(crest_a, crest_b)
        texture += shore_foam + wash_up + ripples + wet_sand
        texture = min(1.0, texture)
        vignette_x = abs(x - 0.5) * 2.0
        vignette_y = abs(y - 0.5) * 2.0
        vignette = max(0.0, 1.0 - 0.18 * vignette_x * vignette_x - 0.10 * vignette_y * vignette_y)
        level = max(0.0, min(1.0, texture * vignette * options.contrast))
        char = wave_char(level)
        color = gray_fg(level)
        if last_color != color:
          line.append(f"\x1b[38;5;{color}m")
          last_color = color
        line.append(char)
      line.append(RESET)
      rows.append("".join(line))
    return "\n".join(rows)
