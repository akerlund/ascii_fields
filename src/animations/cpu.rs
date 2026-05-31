use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, render_glyph_field, FieldStyle};

const STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "copper" };

const STAGES: &[&str] = &["IF", "ID", "EX", "MEM", "WB"];
const PROGRAM: &[&str] = &["LD", "ADD", "MUL", "ST", "BR", "XOR", "LD", "SUB"];

fn make_putters<'a>(grid: &'a mut Vec<f64>, glyphs: &'a mut Vec<char>, w: usize, h: usize)
  -> impl FnMut(i64, i64, f64, char) + 'a {
  move |col: i64, row: i64, value: f64, ch: char| {
    if col >= 0 && (col as usize) < w && row >= 0 && (row as usize) < h {
      let idx = (row as usize) * w + (col as usize);
      if value >= grid[idx] { grid[idx] = value; glyphs[idx] = ch; }
    }
  }
}

pub struct Cpu;

impl Animation for Cpu {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let w = ctx.width; let h = ctx.height;
    if w == 0 || h == 0 { return; }
    let mut grid = vec![0.0_f64; w * h];
    let mut glyphs = vec![' '; w * h];
    let mut put = make_putters(&mut grid, &mut glyphs, w, h);

    let text = |put: &mut dyn FnMut(i64, i64, f64, char), col: i64, row: i64, value: f64, t: &str| {
      for (i, ch) in t.chars().enumerate() { put(col + i as i64, row, value, ch); }
    };
    let hline = |put: &mut dyn FnMut(i64, i64, f64, char), row: i64, c0: i64, c1: i64, value: f64, glyph: char| {
      if !(0..h as i64).contains(&row) { return; }
      let (lo, hi) = if c0 < c1 { (c0, c1) } else { (c1, c0) };
      let lo = lo.max(0); let hi = hi.min((w - 1) as i64);
      for c in lo..=hi { put(c, row, value, glyph); }
    };
    let vline = |put: &mut dyn FnMut(i64, i64, f64, char), col: i64, r0: i64, r1: i64, value: f64, glyph: char| {
      if !(0..w as i64).contains(&col) { return; }
      let (lo, hi) = if r0 < r1 { (r0, r1) } else { (r1, r0) };
      let lo = lo.max(0); let hi = hi.min((h - 1) as i64);
      for r in lo..=hi { put(col, r, value, glyph); }
    };
    let bx = |put: &mut dyn FnMut(i64, i64, f64, char),
              c0: i64, r0: i64, ww: i64, hh: i64, label: &str, value: f64| {
      let c1 = (c0 + ww - 1).min((w - 1) as i64);
      let r1 = (r0 + hh - 1).min((h - 1) as i64);
      hline(put, r0, c0, c1, value, '=');
      hline(put, r1, c0, c1, value, '=');
      vline(put, c0, r0, r1, value, '|');
      vline(put, c1, r0, r1, value, '|');
      let max_n = (c1 - c0 - 1).max(0);
      for (i, ch) in label.chars().take(max_n as usize).enumerate() {
        put(c0 + 1 + i as i64, r0, value + 0.08, ch);
      }
    };

    let margin = (w / 30).max(1) as i64;
    let stage_w = ((w as i64 - margin * 2) / 7).max(6);
    let stage_h = ((h / 5) as i64).max(4);
    let top = ((h / 6) as i64).max(1);
    let gap = (((w as i64 - margin * 2 - stage_w * STAGES.len() as i64) / (STAGES.len() as i64 - 1).max(1)).max(1)) as i64;
    let mut stage_pos: Vec<(i64, i64)> = Vec::new();
    for (idx, name) in STAGES.iter().enumerate() {
      let col = margin + idx as i64 * (stage_w + gap);
      stage_pos.push((col, top));
      bx(&mut put, col, top, stage_w, stage_h, name, 0.38);
      if idx > 0 {
        hline(&mut put, top + stage_h / 2, stage_pos[idx - 1].0 + stage_w, col, 0.28, '-');
      }
    }
    let reg_col = margin;
    let reg_row = ((h as i64).saturating_sub(6)).min(top + stage_h + 3);
    let reg_w = ((w / 5) as i64).max(12);
    let reg_h = ((h / 4) as i64).max(5);
    let alu_col = (reg_col + reg_w + 3).max(w as i64 / 2 - stage_w / 2);
    let mem_col = ((w as i64) - reg_w - margin).min(alu_col + stage_w + 4);
    bx(&mut put, reg_col, reg_row, reg_w, reg_h, "REGS", 0.36);
    bx(&mut put, alu_col, reg_row, stage_w, reg_h, "ALU", 0.42);
    bx(&mut put, mem_col, reg_row, reg_w, reg_h, "CACHE", 0.36);

    let active_reg = (ctx.elapsed * 1.5) as i64 % ((reg_h - 2).max(2) as i64);
    for idx in 0..(reg_h - 2).max(2) {
      let row = reg_row + 1 + idx;
      let hot = if idx == active_reg { 0.95 } else { 0.33 };
      let val = (idx * 7 + (ctx.elapsed * 3.0) as i64) & 31;
      text(&mut put, reg_col + 2, row, hot, &format!("R{}:{:02X}", idx, val));
    }
    let bus_y = ((h as i64 - 2)).min(reg_row + reg_h + 1);
    hline(&mut put, bus_y, margin, w as i64 - margin - 1, 0.22, '=');
    for c in [reg_col + reg_w, alu_col, alu_col + stage_w, mem_col] {
      vline(&mut put, c, top + stage_h, bus_y, 0.22, '|');
      put(c, bus_y, 0.55, '+');
    }

    let cycle = ctx.elapsed * 1.55;
    let tick = cycle as i64;
    let frac = cycle - tick as f64;
    for slot in 0..7_i64 {
      let instr_idx = tick - slot;
      if instr_idx < -1 { continue; }
      let stage_idx = slot as usize;
      if stage_idx >= STAGES.len() { continue; }
      let instr = PROGRAM[(instr_idx as usize) % PROGRAM.len()];
      let (col, row) = stage_pos[stage_idx];
      let hot = 0.65 + 0.35 * (ctx.elapsed * 3.0 + slot as f64).sin();
      text(&mut put, col + 2, row + stage_h / 2, hot, instr);
      if frac > 0.50 && stage_idx + 1 < STAGES.len() {
        let c0 = col + stage_w;
        let c1 = stage_pos[stage_idx + 1].0;
        let pulse_f = (frac - 0.50) * 2.0;
        let pcol = (c0 as f64 + (c1 - c0) as f64 * pulse_f).round() as i64;
        put(pcol, row + stage_h / 2, 1.0, '>');
      }
    }

    let cyc = format!("cyc:{:04}", tick);
    text(&mut put, (w as i64 - cyc.len() as i64 - 1).max(0), (top - 1).max(0), 0.85, &cyc);

    let hazard = matches!(PROGRAM[(tick as usize) % PROGRAM.len()], "LD" | "BR");
    if hazard {
      text(&mut put, stage_pos[1].0 + 1, (top - 1).max(0), 0.95, "STALL");
      for c in stage_pos[1].0..stage_pos[2].0 + stage_w {
        if c.rem_euclid(2) == tick.rem_euclid(2) {
          put(c, top + stage_h + 1, 0.62, '!');
        }
      }
      let mem_col_pos = stage_pos[3].0;
      let ex_col = stage_pos[2].0 + stage_w - 1;
      let arrow_row = top + stage_h / 2;
      for c in ex_col..mem_col_pos { put(c, arrow_row, 0.88, '<'); }
    }

    let routes: &[((i64, i64), (i64, i64), char, f64)] = &[
      ((reg_col + reg_w, reg_row + reg_h / 2), (alu_col, reg_row + reg_h / 2), '@', 0.00),
      ((alu_col + stage_w, reg_row + reg_h / 2), (mem_col, reg_row + reg_h / 2), '*', 0.22),
      ((mem_col, bus_y), (reg_col + reg_w, bus_y), '+', 0.48),
    ];
    for &(a, b, glyph, offset) in routes {
      let (x0, y0) = a; let (x1, y1) = b;
      let pulse = ((ctx.elapsed * 0.65 + offset) % 1.0 + 1.0) % 1.0;
      let steps = 6_i64;
      for tail in 0..steps {
        let f = pulse - tail as f64 * 0.035;
        if f < 0.0 { continue; }
        put(
          (x0 as f64 + (x1 - x0) as f64 * f).round() as i64,
          (y0 as f64 + (y1 - y0) as f64 * f).round() as i64,
          1.0 - tail as f64 * 0.12,
          if tail == 0 { glyph } else { '.' },
        );
      }
    }
    drop(put);
    let contrast = ctx.options.contrast;
    for v in grid.iter_mut() { *v = clamp(*v * contrast); }
    render_glyph_field(ctx, &grid, &glyphs, &STYLE, out);
  }
}
