import math

from ..core import Animation, clamp, render_field, smoothstep


PLUMES = (
  (0.2, 0.47, 1.38, 0.055),
  (1.6, -0.31, 1.22, 0.045),
  (2.8, 0.22, 1.50, 0.060),
  (4.3, -0.18, 1.32, 0.050),
  (5.2, 0.36, 1.18, 0.040),
)

LOOPS = ((0.8, 0.20, 0.34), (2.4, -0.16, 0.28), (3.7, 0.13, 0.42))


class FlaresAnimation(Animation):
  thresholds = [
    (0.13, " "), (0.22, "."), (0.31, ":"), (0.42, "-"), (0.54, "="),
    (0.67, "+"), (0.80, "*"), (0.92, "#"), (1.01, "%"),
  ]

  def render(self, width, height, elapsed, phase, options):
    radius = max(1.0, min(width, height * 2) * 0.34)
    t = elapsed * 0.9
    sin_t023 = 0.10 * math.sin(t * 0.23)
    grid = []
    for row in range(height):
      py = ((row - (height - 1) * 0.5) * 2.0) / radius
      line = []
      for col in range(width):
        px = (col - (width - 1) * 0.5) / radius
        r = math.hypot(px, py)
        theta = math.atan2(py, px) + sin_t023
        limb = smoothstep(1.04, 0.90, r)
        corona = max(0.0, math.exp(-3.3 * max(0.0, r - 0.92)) - 0.12)
        surface = limb * (
          0.20
          + 0.13 * abs(math.sin(9.0 * theta + 1.4 * math.sin(3.0 * theta - 0.4 * t)))
          + 0.09 * abs(math.sin(17.0 * theta + 0.55 * t))
        )
        radial = max(0.0, r - 0.72)
        plume = 0.0
        for offset, speed, reach, width_factor in PLUMES:
          center = offset + speed * t + 0.28 * math.sin(1.7 * radial + t * 0.33 + offset)
          angular_delta = math.atan2(math.sin(theta - center), math.cos(theta - center))
          strand = math.exp(-(angular_delta * angular_delta) / width_factor)
          height_gate = smoothstep(0.76, 1.02, r) * smoothstep(reach, 0.94, r)
          texture = 0.60 + 0.40 * abs(math.sin(18.0 * radial - 2.4 * t + offset))
          plume += strand * height_gate * texture
        loops = 0.0
        for offset, speed, arch_height in LOOPS:
          center = offset + speed * t
          angular_delta = math.atan2(math.sin(theta - center), math.cos(theta - center))
          arch = 1.00 + arch_height * math.sin(max(0.0, 1.0 - abs(angular_delta) / 0.55) * math.pi)
          loop_line = math.exp(-((r - arch) ** 2) / 0.0025)
          loop_span = smoothstep(0.62, 0.08, abs(angular_delta))
          pulse = 0.55 + 0.45 * abs(math.sin(13.0 * angular_delta + 1.5 * t + offset))
          loops += loop_line * loop_span * pulse
        sparks = 0.10 * max(0.0, math.sin(37.0 * r + 11.0 * theta - 2.2 * t))
        exterior = smoothstep(0.82, 1.02, r)
        texture = surface + exterior * (0.22 * corona + 0.56 * plume + 0.52 * loops + sparks)
        vignette = max(0.0, 1.0 - 0.07 * (abs(px) + abs(py)))
        line.append(clamp(texture * vignette * options.contrast))
      grid.append(line)
    return render_field(width, height, grid, options, self)
