import math

from ..core import Animation, clamp, render_field, smoothstep


class AreasAnimation(Animation):
  thresholds = [
    (0.20, " "), (0.28, "."), (0.36, ":"), (0.47, "-"), (0.58, "="),
    (0.70, "+"), (0.82, "*"), (0.93, "#"), (1.01, "%"),
  ]

  def render(self, width, height, elapsed, phase, options):
    radius_x = max(1.0, width * 0.48)
    radius_y = max(1.0, height * 0.43)
    density = max(0.35, options.scale * 0.62)
    t = elapsed * 1.55
    sin_t = math.sin(t)
    drift_x = 0.24 * sin_t + 0.10 * math.sin(elapsed * 0.47)
    drift_y = 0.11 * math.cos(t * 0.83) + 0.06 * math.sin(elapsed * 0.31)
    col_step = 2 if width >= 40 else 1
    cols = [
      (
        min(col_step, width - col),
        ((col + (col_step - 1) * 0.5) - (width - 1) * 0.5) / radius_x,
      )
      for col in range(0, width, col_step)
    ]
    grid = []
    for row in range(height):
      py = (row - (height - 1) * 0.5) / radius_y
      py2 = py * py
      y = (py + drift_y) * density
      sin_y2 = math.sin(1.25 * y + 0.17 * t)
      sin_y27 = math.sin(1.85 * y + 0.42 * t)
      sin_y31 = math.sin(18.0 * y)
      line = []
      for repeat, px in cols:
        r2 = px * px + py2
        if r2 >= 1.0:
          line.extend([0.0] * repeat)
          continue
        x = (px * 1.02 + drift_x) * density
        columns_a = smoothstep(0.42, 0.88, abs(math.sin(4.5 * x + 1.1 * sin_y27)))
        columns_b = smoothstep(0.50, 0.94, abs(math.sin(7.0 * x - 1.15 * y + 0.38 * t)))
        broad_a = smoothstep(
          -0.58, 0.50,
          sin_y2
          + 0.72 * math.sin(1.65 * x - 0.95 * y + 0.50 * t)
          + 0.48 * math.cos(2.35 * x + 0.45 * y - 0.25 * t),
        )
        broad_b = smoothstep(
          -0.62, 0.56,
          math.sin(1.15 * x + 1.55 * y - 0.68 * t)
          + 0.54 * math.cos(2.05 * x - 0.55 * y + 0.43 * t),
        )
        broad_c = smoothstep(
          -0.50, 0.64,
          math.cos(0.85 * x - 1.95 * y + 0.34 * t)
          + 0.46 * math.sin(2.70 * x + 0.25 * y - 0.58 * t),
        )
        broad = min(1.0, 0.68 * broad_a + 0.46 * broad_b + 0.36 * broad_c)
        holes = smoothstep(0.48, 0.92, abs(math.sin(3.4 * x - 3.0 * y + 0.65 * sin_t)))
        stripes = smoothstep(0.22, 0.84, abs(sin_y31 + 0.18 * math.sin(2.2 * x)))
        region = broad * (0.45 + 0.45 * max(columns_a, columns_b)) * (1.0 - 0.45 * holes)
        texture = min(1.0, 0.16 + 0.07 * columns_a + 0.05 * stripes
                      + region * (0.38 + 0.24 * stripes + 0.18 * columns_b))
        edge_fade = max(0.0, 1.0 - r2 * r2)
        shadow = 0.62 + 0.38 * math.sqrt(max(0.0, 1.0 - r2))
        level = clamp(texture * edge_fade * shadow * options.contrast)
        line.extend([level] * repeat)
      grid.append(line)
    return render_field(width, height, grid, options, self)
