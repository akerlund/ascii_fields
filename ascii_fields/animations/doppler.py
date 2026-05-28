import math

from ..core import Animation, BLACK_BG, RESET, clamp, density_char, gray_fg, resolve_theme


class _Front:
  __slots__ = ("x", "y", "vx", "vy", "t0")

  def __init__(self, x, y, vx, vy, t0):
    self.x = x
    self.y = y
    self.vx = vx
    self.vy = vy
    self.t0 = t0


def _shift_color(shift):
  # shift: 0 = redshift, 0.5 = none, 1 = blueshift
  if shift < 0.5:
    t = shift / 0.5
    return (255, int(70 + 185 * t), int(70 + 185 * t))
  t = (shift - 0.5) / 0.5
  return (int(255 - 215 * t), int(255 - 150 * t), 255)


class DopplerAnimation(Animation):
  """The radial-velocity wobble of a star tugged by an orbiting exoplanet. The
  star circles the shared barycentre and its light ripples outward; the side it
  moves toward is blueshifted, the side it recedes from is redshifted. With a
  truecolor terminal the shift is shown in colour."""

  thresholds = [
    (0.10, " "), (0.22, "."), (0.34, ":"), (0.46, "-"), (0.58, "="),
    (0.70, "+"), (0.82, "*"), (0.92, "#"), (1.01, "@"),
  ]

  C = 0.42                  # wave speed (normalised units / s)
  EMIT_DT = 0.14

  def __init__(self):
    self._fronts = []
    self._next = 0.0
    self._last = 0.0

  def render(self, width, height, elapsed, phase, options):
    if elapsed < self._last:
      self._fronts = []
      self._next = 0.0
    self._last = elapsed

    ax = width / max(1, height * 2.0)
    omega = 1.4
    r_star = 0.18
    r_planet = 0.62
    # star wobble and (opposite-phase) planet
    sx = r_star * math.cos(omega * elapsed)
    sy = r_star * math.sin(omega * elapsed)
    svx = -r_star * omega * math.sin(omega * elapsed)
    svy = r_star * omega * math.cos(omega * elapsed)
    px = -r_planet * math.cos(omega * elapsed)
    py = -r_planet * math.sin(omega * elapsed)

    while elapsed >= self._next:
      tt = self._next
      ssx = r_star * math.cos(omega * tt)
      ssy = r_star * math.sin(omega * tt)
      vvx = -r_star * omega * math.sin(omega * tt)
      vvy = r_star * omega * math.cos(omega * tt)
      self._fronts.append(_Front(ssx, ssy, vvx, vvy, tt))
      self._next += self.EMIT_DT
    self._fronts = [f for f in self._fronts if elapsed - f.t0 < 5.0]

    mono = resolve_theme(options, "scene") == "mono"
    bright = options.brightness
    contrast = options.contrast
    rows = []
    for row in range(height):
      cy = (row / max(1, height - 1) - 0.5) * 2.0
      line = [BLACK_BG]
      last = None
      for col in range(width):
        cx = (col / max(1, width - 1) - 0.5) * 2.0 * ax
        intensity = 0.0
        dop_acc = 0.0
        for f in self._fronts:
          radius = self.C * (elapsed - f.t0)
          dx = cx - f.x
          dy = cy - f.y
          dist = math.hypot(dx, dy) or 1e-4
          b = math.exp(-((dist - radius) ** 2) / 0.0010)
          if b > 0.001:
            intensity += b
            dop = (f.vx * dx + f.vy * dy) / (dist * self.C)
            dop_acc += b * dop
        # star and planet markers
        intensity += 1.0 * math.exp(-(((cx - sx) ** 2 + (cy - sy) ** 2)) / 0.0016)
        intensity += 0.5 * math.exp(-(((cx - px) ** 2 + (cy - py) ** 2)) / 0.0010)
        value = clamp(intensity * contrast)
        ch = density_char(value, self.thresholds)
        if value <= 0.001:
          line.append(" ")
          last = None
          continue
        if mono:
          color = gray_fg(value, 234, 255, bright)
          if color != last:
            line.append(f"\x1b[38;5;{color}m")
            last = color
        else:
          shift = clamp(0.5 + 0.5 * (dop_acc / intensity if intensity else 0.0) * 1.6)
          r, g, b = _shift_color(shift)
          rgb = (int(r * value), int(g * value), int(b * value))
          if rgb != last:
            line.append(f"\x1b[38;2;{rgb[0]};{rgb[1]};{rgb[2]}m")
            last = rgb
        line.append(ch)
      line.append(RESET)
      rows.append("".join(line))
    return "\n".join(rows)
