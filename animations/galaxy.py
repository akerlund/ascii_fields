import math

from core import Animation, BLACK_BG, RESET, density_char, gray_fg, smoothstep, star_noise


GALAXY_CHAR_THRESHOLDS = [(0.11, " "), (0.19, "."), (0.29, ":"), (0.40, "-"), (0.52, "="), (0.65, "+"), (0.78, "*"), (0.91, "#"), (1.01, "%")]


def galaxy_char(level):
  return density_char(level, GALAXY_CHAR_THRESHOLDS)


class GalaxyAnimation(Animation):
  def render(self, width, height, elapsed, phase, options):
    radius = max(1.0, min(width, height * 2) * 0.47)
    t = elapsed * 0.18
    rows = []
    for row in range(height):
      line = [BLACK_BG]
      last_color = None
      py = ((row - (height - 1) * 0.5) * 2.0) / radius
      for col in range(width):
        px = (col - (width - 1) * 0.5) / radius
        tilted_y = py / 0.62
        spin = t * (0.45 + 0.55 / (1.0 + 10.0 * (px * px + tilted_y * tilted_y)))
        cs = math.cos(spin)
        sn = math.sin(spin)
        gx = px * cs - tilted_y * sn
        gy = px * sn + tilted_y * cs
        r = math.hypot(gx, gy)
        theta = math.atan2(gy, gx)
        core = math.exp(-18.0 * r * r)
        halo = math.exp(-2.8 * r)
        disk = smoothstep(1.18, 0.08, r)
        twist = theta + 4.4 * math.log(1.0 + r * 3.4) - t * 1.35
        arm_a = smoothstep(0.84, 0.985, math.cos(2.0 * twist))
        arm_b = smoothstep(0.80, 0.990, math.cos(2.0 * twist + math.pi))
        arm_c = smoothstep(0.86, 0.995, math.cos(3.0 * twist - 0.6))
        arms = max(arm_a, 0.82 * arm_b, 0.52 * arm_c)
        arm_fade = smoothstep(0.03, 0.18, r) * smoothstep(1.16, 0.42, r)
        dust = smoothstep(
          0.20,
          0.86,
          abs(math.sin(14.0 * theta + 9.0 * r - 0.7 * t))
          + 0.18 * math.sin(23.0 * r + 3.0 * theta),
        )
        mottled = 0.70 + 0.30 * abs(math.sin(33.0 * r + 7.0 * theta + t))
        stars = 0.0
        if star_noise(col, row) > 0.993:
          stars = 0.65 + 0.35 * math.sin(elapsed * 2.1 + col * 0.7)
        elif star_noise(col + 19, row - 7) > 0.985:
          stars = 0.20
        texture = 0.54 * core + 0.18 * halo * disk
        texture += 0.62 * arms * arm_fade * mottled * (1.0 - 0.40 * dust)
        texture += stars * (1.0 - smoothstep(0.0, 0.45, core))
        vignette = max(0.0, 1.0 - 0.10 * abs(px) - 0.16 * abs(py))
        level = max(0.0, min(1.0, texture * vignette * options.contrast))
        char = galaxy_char(level)
        color = gray_fg(level)
        if last_color != color:
          line.append(f"\x1b[38;5;{color}m")
          last_color = color
        line.append(char)
      line.append(RESET)
      rows.append("".join(line))
    return "\n".join(rows)
