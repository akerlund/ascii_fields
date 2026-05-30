import math

from ..core import Animation, clamp, render_glyph_field


STAGES = ("IF", "ID", "EX", "MEM", "WB")
PROGRAM = ("LD", "ADD", "MUL", "ST", "BR", "XOR", "LD", "SUB")


class CPUAnimation(Animation):
  """A simplified superscalar CPU view: instructions move through pipeline
  stages, registers feed an ALU, memory responds, and forwarding/hazard signals
  pulse over the data paths."""

  default_theme = "copper"

  def render(self, width, height, elapsed, phase, options):
    grid = [[0.0] * width for _ in range(height)]
    glyphs = [[" "] * width for _ in range(height)]
    if width <= 0 or height <= 0:
      return ""

    def put(col, row, value, glyph):
      if 0 <= col < width and 0 <= row < height and value >= grid[row][col]:
        grid[row][col] = value
        glyphs[row][col] = glyph

    def text(col, row, value, value_text):
      for idx, ch in enumerate(value_text):
        put(col + idx, row, value, ch)

    def hline(row, c0, c1, value=0.24, glyph="-"):
      if not 0 <= row < height:
        return
      lo, hi = sorted((max(0, c0), min(width - 1, c1)))
      for col in range(lo, hi + 1):
        put(col, row, value, glyph)

    def vline(col, r0, r1, value=0.24, glyph="|"):
      if not 0 <= col < width:
        return
      lo, hi = sorted((max(0, r0), min(height - 1, r1)))
      for row in range(lo, hi + 1):
        put(col, row, value, glyph)

    def box(c0, r0, w, h, label, value=0.42):
      c1 = min(width - 1, c0 + w - 1)
      r1 = min(height - 1, r0 + h - 1)
      hline(r0, c0, c1, value, "=")
      hline(r1, c0, c1, value, "=")
      vline(c0, r0, r1, value, "|")
      vline(c1, r0, r1, value, "|")
      text(c0 + 1, r0, value + 0.08, label[:max(0, w - 2)])

    margin = max(1, width // 30)
    stage_w = max(6, (width - margin * 2) // 7)
    stage_h = max(4, height // 5)
    top = max(1, height // 6)
    gap = max(1, (width - margin * 2 - stage_w * len(STAGES)) // max(1, len(STAGES) - 1))
    stage_pos = []
    for idx, name in enumerate(STAGES):
      col = margin + idx * (stage_w + gap)
      stage_pos.append((col, top))
      box(col, top, stage_w, stage_h, name, 0.38)
      if idx:
        hline(top + stage_h // 2, stage_pos[idx - 1][0] + stage_w, col, 0.28)

    reg_col = margin
    reg_row = min(height - 6, top + stage_h + 3)
    reg_w = max(12, width // 5)
    reg_h = max(5, height // 4)
    alu_col = max(reg_col + reg_w + 3, width // 2 - stage_w // 2)
    alu_row = reg_row
    mem_col = min(width - reg_w - margin, alu_col + stage_w + 4)
    box(reg_col, reg_row, reg_w, reg_h, "REGS", 0.36)
    box(alu_col, alu_row, stage_w, reg_h, "ALU", 0.42)
    box(mem_col, reg_row, reg_w, reg_h, "CACHE", 0.36)

    # Pick a register to mark 'active' this cycle so the eye has something
    # to follow in the REGS box; cycles through R0..R(N-1) at ~1.5 Hz.
    active_reg = int(elapsed * 1.5) % max(2, reg_h - 2)
    for idx in range(max(2, reg_h - 2)):
      row = reg_row + 1 + idx
      hot = 0.95 if idx == active_reg else 0.33
      text(reg_col + 2, row, hot, f"R{idx}:{(idx * 7 + int(elapsed * 3)) & 31:02X}")

    bus_y = min(height - 2, reg_row + reg_h + 1)
    hline(bus_y, margin, width - margin - 1, 0.22, "=")
    for col in (reg_col + reg_w, alu_col, alu_col + stage_w, mem_col):
      vline(col, top + stage_h, bus_y, 0.22)
      put(col, bus_y, 0.55, "+")

    cycle = elapsed * 1.55
    tick = int(cycle)
    frac = cycle - tick
    for slot in range(7):
      instr_idx = tick - slot
      if instr_idx < -1:
        continue
      stage_idx = slot
      if 0 <= stage_idx < len(STAGES):
        instr = PROGRAM[instr_idx % len(PROGRAM)]
        col, row = stage_pos[stage_idx]
        hot = 0.65 + 0.35 * math.sin(elapsed * 3.0 + slot)
        text(col + 2, row + stage_h // 2, hot, instr)
        if frac > 0.50 and stage_idx < len(STAGES) - 1:
          c0 = col + stage_w
          c1 = stage_pos[stage_idx + 1][0]
          pulse_f = (frac - 0.50) * 2.0
          pcol = round(c0 + (c1 - c0) * pulse_f)
          put(pcol, row + stage_h // 2, 1.0, ">")

    # Cycle counter pinned to top-right.
    cyc_label = f"cyc:{tick:04d}"
    text(max(0, width - len(cyc_label) - 1), max(0, top - 1), 0.85, cyc_label)

    hazard = PROGRAM[tick % len(PROGRAM)] in ("LD", "BR")
    if hazard:
      text(stage_pos[1][0] + 1, max(0, top - 1), 0.95, "STALL")
      for col in range(stage_pos[1][0], stage_pos[2][0] + stage_w):
        if col % 2 == tick % 2:
          put(col, top + stage_h + 1, 0.62, "!")
      # Forwarding arrow from MEM back to EX during a stall.
      mem_col = stage_pos[3][0]
      ex_col = stage_pos[2][0] + stage_w - 1
      arrow_row = top + stage_h // 2
      for col in range(ex_col, mem_col):
        put(col, arrow_row, 0.88, "<")

    # Register, ALU, cache pulses.
    routes = (
      ((reg_col + reg_w, reg_row + reg_h // 2), (alu_col, alu_row + reg_h // 2), "@", 0.00),
      ((alu_col + stage_w, alu_row + reg_h // 2), (mem_col, reg_row + reg_h // 2), "*", 0.22),
      ((mem_col, bus_y), (reg_col + reg_w, bus_y), "+", 0.48),
    )
    for (a, b, glyph, offset) in routes:
      x0, y0 = a
      x1, y1 = b
      pulse = (elapsed * 0.65 + offset) % 1.0
      steps = 6
      for tail in range(steps):
        f = pulse - tail * 0.035
        if f < 0.0:
          continue
        put(round(x0 + (x1 - x0) * f), round(y0 + (y1 - y0) * f),
            1.0 - tail * 0.12, glyph if tail == 0 else ".")

    contrast = options.contrast
    return render_glyph_field(
      width, height,
      [[clamp(value * contrast) for value in row] for row in grid],
      glyphs, options, self
    )
