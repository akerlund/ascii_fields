import math

from core import Animation, BLACK_BG, RESET, density_char, gray_fg, smoothstep, star_noise


BLACK_HOLE_CHAR_THRESHOLDS = [(0.10, " "), (0.18, "."), (0.28, ":"), (0.39, "-"), (0.51, "="), (0.64, "+"), (0.78, "*"), (0.91, "#"), (1.01, "%")]


def black_hole_char(level):
  return density_char(level, BLACK_HOLE_CHAR_THRESHOLDS)


def accretion_band(x, y, radius, thickness, t, density, boost):
  disk_radius = math.hypot(x, y)
  disk_theta = math.atan2(y, x)
  ring = math.exp(-((disk_radius - radius) ** 2) / thickness)
  azimuth = 0.62 + 0.38 * math.sin(9.0 * disk_theta - 2.2 * t)
  turbulence = 0.72 + 0.28 * abs(math.sin(31.0 * disk_radius * density + 5.0 * disk_theta - 3.4 * t))
  return ring * azimuth * turbulence * boost


class BlackHoleAnimation(Animation):
  def render(self, width, height, elapsed, phase, options):
    radius = max(1.0, min(width, height * 2) * 0.44)
    density = max(0.45, options.scale)
    t = elapsed * 0.55
    rows = []
    for row in range(height):
      line = [BLACK_BG]
      last_color = None
      py = ((row - (height - 1) * 0.5) * 2.0) / radius
      for col in range(width):
        px = (col - (width - 1) * 0.5) / radius
        r = math.hypot(px, py)
        lens = 0.18 / (r * r + 0.08)
        bent_y = py + lens * (1.0 if py >= 0.0 else -1.0)
        disk_y = bent_y / 0.18
        front_y = (py + 0.12) / 0.16
        back_y = (py - 0.18 - 0.50 * math.exp(-9.0 * px * px)) / 0.13
        front = accretion_band(px, front_y, 0.92, 0.035, t, density, 0.95)
        upper_arc = accretion_band(px, back_y, 0.98, 0.050, t * 0.85 + 1.7, density, 0.75)
        lower_lens = 0.42 * accretion_band(px, disk_y, 1.08, 0.060, -t * 0.55, density, 0.55)
        ring_glow = math.exp(-((r - 0.34) ** 2) / 0.005) * 0.38
        photon_ring = math.exp(-((r - 0.43) ** 2) / 0.0018) * 0.55
        shadow = smoothstep(0.41, 0.34, r)
        hole_cut = smoothstep(0.48, 0.39, r)
        doppler = 0.74 + 0.42 * smoothstep(-0.45, 0.65, px)
        disk = (front * doppler + upper_arc + lower_lens) * (1.0 - 0.70 * hole_cut)
        halo = 0.13 * math.exp(-2.5 * max(0.0, r - 0.36))
        background_stars = 0.0
        if star_noise(col + 101, row - 31) > 0.996:
          background_stars = 0.45 + 0.25 * math.sin(elapsed * 1.7 + col)
        texture = disk + photon_ring + ring_glow + halo + background_stars
        texture *= 1.0 - 0.96 * shadow
        vignette = max(0.0, 1.0 - 0.08 * abs(px) - 0.12 * abs(py))
        level = max(0.0, min(1.0, texture * vignette * options.contrast))
        char = black_hole_char(level)
        color = gray_fg(level)
        if last_color != color:
          line.append(f"\x1b[38;5;{color}m")
          last_color = color
        line.append(char)
      line.append(RESET)
      rows.append("".join(line))
    return "\n".join(rows)
