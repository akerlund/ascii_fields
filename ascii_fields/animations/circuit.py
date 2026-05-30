import math

from ..core import Animation, clamp, render_glyph_field


class CircuitAnimation(Animation):
  """A stylised circuit board with varied modules, traces, buses, LEDs and data
  pulses moving through several routes."""

  default_theme = "amber"

  def render(self, width, height, elapsed, phase, options):
    if width <= 0 or height <= 0:
      return ""
    # At very small terminals the labelled boxes overlap into garbage; render
    # just buses + moving pulses, which still reads as 'circuit'.
    if width < 50 or height < 12:
      return self._render_small(width, height, elapsed, options)

    grid = [[0.0] * width for _ in range(height)]
    glyphs = [[" "] * width for _ in range(height)]

    def put(col, row, value, glyph):
      if 0 <= col < width and 0 <= row < height and value >= grid[row][col]:
        grid[row][col] = value
        glyphs[row][col] = glyph

    def hline(row, c0, c1, value=0.25, glyph="-"):
      if not 0 <= row < height:
        return
      lo, hi = sorted((max(0, c0), min(width - 1, c1)))
      for col in range(lo, hi + 1):
        put(col, row, value, glyph)

    def vline(col, r0, r1, value=0.25, glyph="|"):
      if not 0 <= col < width:
        return
      lo, hi = sorted((max(0, r0), min(height - 1, r1)))
      for row in range(lo, hi + 1):
        put(col, row, value, glyph)

    def box(c0, r0, w, h, label, value=0.50):
      c1 = min(width - 1, c0 + w - 1)
      r1 = min(height - 1, r0 + h - 1)
      if c0 >= width or r0 >= height:
        return
      hline(r0, c0, c1, value, "=")
      hline(r1, c0, c1, value, "=")
      vline(c0, r0, r1, value, "|")
      vline(c1, r0, r1, value, "|")
      for i, ch in enumerate(label[:max(0, c1 - c0 - 1)]):
        put(c0 + 1 + i, (r0 + r1) // 2, value + 0.12, ch)

    def diode(col, row, phase_offset):
      beat = 0.45 + 0.55 * max(0.0, math.sin(elapsed * 4.2 + phase_offset))
      put(col, row, 0.25 + 0.75 * beat, "o" if beat < 0.82 else "@")

    top = max(1, height // 6)
    bottom = min(height - 2, height - height // 7)
    left = max(1, width // 18)
    right = min(width - 2, width - width // 18)
    mid = width // 2
    rows = [top, height // 3, height // 2, (height * 2) // 3, bottom]

    # Board-wide backplane buses.
    for idx, row in enumerate(rows):
      hline(row, left, right, 0.20 + idx * 0.025, "=" if idx == 2 else "-")
    for idx, col in enumerate((left, width // 4, mid, (width * 3) // 4, right)):
      vline(col, top, bottom, 0.18 + idx * 0.02)
      for row in rows:
        put(col, row, 0.55, "+")

    cpu = (max(2, width // 7), max(2, height // 3 - 2), max(8, width // 7), max(5, height // 4))
    mem = (max(2, width // 2 - width // 12), max(1, height // 7), max(10, width // 6), max(4, height // 6))
    io = (min(width - max(10, width // 8) - 2, width - width // 5), max(2, height // 2 - 2),
          max(8, width // 8), max(5, height // 4))
    rf = (max(2, width // 3), min(height - max(4, height // 6) - 1, bottom - 2),
          max(9, width // 7), max(4, height // 6))
    box(*cpu, "CPU")
    box(*mem, "RAM", value=0.44)
    box(*io, "IO", value=0.48)
    box(*rf, "ADC", value=0.42)

    # Fan-out traces, not just one central corridor.
    for pin in range(cpu[1] + 1, cpu[1] + cpu[3] - 1):
      if pin % 2 == 0:
        hline(pin, cpu[0] + cpu[2], mid, 0.26)
    for pin in range(io[1] + 1, io[1] + io[3] - 1):
      if pin % 2 == 1:
        hline(pin, mid, io[0], 0.24)
    for col in range(mem[0] + 1, mem[0] + mem[2] - 1, 2):
      vline(col, mem[1] + mem[3], rows[2], 0.22)
    for col in range(rf[0] + 1, rf[0] + rf[2] - 1, 2):
      vline(col, rows[2], rf[1], 0.22)

    for idx, col in enumerate(range(left + 3, right, max(5, width // 12))):
      diode(col, max(1, height - 2), idx * 0.8)

    paths = [
      ((left, rows[0]), (right, rows[0]), 3.8, 0.00, ">"),
      ((right, rows[-1]), (left, rows[-1]), 4.4, 0.19, "<"),
      ((mid, top), (mid, bottom), 3.2, 0.35, "v"),
      ((cpu[0] + cpu[2], cpu[1] + 1), (io[0], io[1] + 1), 2.7, 0.48, "*"),
      ((mem[0] + mem[2] // 2, mem[1] + mem[3]), (rf[0] + rf[2] // 2, rf[1]), 3.5, 0.66, "#"),
      ((left, rows[2]), (right, rows[2]), 2.4, 0.82, "@"),
    ]
    for path_idx, (a, b, speed, offset, head) in enumerate(paths):
      x0, y0 = a
      x1, y1 = b
      pulse = (elapsed / speed + offset) % 1.0
      for tail in range(10):
        f = pulse - tail * 0.018
        if f < 0.0:
          f += 1.0
        col = round(x0 + (x1 - x0) * f)
        row = round(y0 + (y1 - y0) * f)
        value = 1.0 - tail * 0.075
        if value > 0.25:
          put(col, row, value, head if tail == 0 else "*")
      # Occasional branch pulse, like a bus transaction hitting a side device.
      if path_idx % 2 == 0:
        branch_col = round(x0 + (x1 - x0) * pulse)
        branch_row = rows[1] if path_idx == 0 else rows[-2]
        vline(branch_col, min(branch_row, round(y0)), max(branch_row, round(y0)), 0.30)
        put(branch_col, branch_row, 0.85, "@")

    # Apply shimmer only to dim infrastructure (traces/buses), not to text
    # labels or bright pulses, so the box labels stay legible.
    shimmer = 0.025 * (0.5 + 0.5 * math.sin(elapsed * 2.7))
    contrast = options.contrast
    out = []
    for row in grid:
      out_row = []
      for value in row:
        if 0.0 < value < 0.55:
          value += shimmer
        out_row.append(clamp(value * contrast))
      out.append(out_row)
    return render_glyph_field(width, height, out, glyphs, options, self)

  def _render_small(self, width, height, elapsed, options):
    grid = [[0.0] * width for _ in range(height)]
    glyphs = [[" "] * width for _ in range(height)]
    bus_rows = [r for r in (height // 4, height // 2, (3 * height) // 4) if 0 <= r < height]
    for row in bus_rows:
      for col in range(width):
        grid[row][col] = 0.22
        glyphs[row][col] = "-" if row != height // 2 else "="
    for offset, glyph, speed in ((0.00, "@", 4.2), (0.33, "*", 3.5), (0.66, "+", 5.0)):
      for row in bus_rows:
        col = int(((elapsed / speed + offset + row * 0.04) % 1.0) * (width - 1))
        if 0 <= col < width:
          grid[row][col] = 1.0
          glyphs[row][col] = glyph
    contrast = options.contrast
    return render_glyph_field(width, height,
                              [[clamp(v * contrast) for v in row] for row in grid],
                              glyphs, options, self)
