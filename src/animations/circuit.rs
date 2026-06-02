use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, render_glyph_field, FieldStyle};

const STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "amber" };

pub struct Circuit;

fn render_small(ctx: &FrameContext, out: &mut String) {
  let w = ctx.width;
  let h = ctx.height;
  let mut grid = vec![0.0_f64; w * h];
  let mut glyphs = vec![' '; w * h];
  let bus_rows: Vec<usize> = [h / 4, h / 2, (3 * h) / 4].iter().cloned().filter(|&r| r < h).collect();
  for &row in &bus_rows {
    for col in 0..w {
      grid[row * w + col] = 0.22;
      glyphs[row * w + col] = if row == h / 2 { '=' } else { '-' };
    }
  }
  for (offset, glyph, speed) in [(0.00, '@', 4.2), (0.33, '*', 3.5), (0.66, '+', 5.0)] {
    for &row in &bus_rows {
      let p = ((ctx.elapsed / speed + offset + row as f64 * 0.04) % 1.0 + 1.0) % 1.0;
      let col = (p * (w as f64 - 1.0)) as usize;
      if col < w {
        grid[row * w + col] = 1.0;
        glyphs[row * w + col] = glyph;
      }
    }
  }
  let contrast = ctx.options.contrast;
  for v in grid.iter_mut() {
    *v = clamp(*v * contrast);
  }
  render_glyph_field(ctx, &grid, &glyphs, &STYLE, out);
}

impl Animation for Circuit {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let w = ctx.width;
    let h = ctx.height;
    if w == 0 || h == 0 {
      return;
    }
    if w < 50 || h < 12 {
      render_small(ctx, out);
      return;
    }
    let mut grid = vec![0.0_f64; w * h];
    let mut glyphs = vec![' '; w * h];
    let put = |grid: &mut Vec<f64>, glyphs: &mut Vec<char>, col: i64, row: i64, value: f64, ch: char| {
      if col >= 0 && (col as usize) < w && row >= 0 && (row as usize) < h {
        let idx = (row as usize) * w + (col as usize);
        if value >= grid[idx] {
          grid[idx] = value;
          glyphs[idx] = ch;
        }
      }
    };
    let hline =
      |grid: &mut Vec<f64>, glyphs: &mut Vec<char>, row: i64, c0: i64, c1: i64, value: f64, glyph: char| {
        if !(0..h as i64).contains(&row) {
          return;
        }
        let (lo, hi) = if c0 < c1 { (c0, c1) } else { (c1, c0) };
        let lo = lo.max(0);
        let hi = hi.min((w - 1) as i64);
        for c in lo..=hi {
          put(grid, glyphs, c, row, value, glyph);
        }
      };
    let vline =
      |grid: &mut Vec<f64>, glyphs: &mut Vec<char>, col: i64, r0: i64, r1: i64, value: f64, glyph: char| {
        if !(0..w as i64).contains(&col) {
          return;
        }
        let (lo, hi) = if r0 < r1 { (r0, r1) } else { (r1, r0) };
        let lo = lo.max(0);
        let hi = hi.min((h - 1) as i64);
        for r in lo..=hi {
          put(grid, glyphs, col, r, value, glyph);
        }
      };
    let bx = |grid: &mut Vec<f64>,
              glyphs: &mut Vec<char>,
              c0: i64,
              r0: i64,
              ww: i64,
              hh: i64,
              label: &str,
              value: f64| {
      let c1 = (c0 + ww - 1).min((w - 1) as i64);
      let r1 = (r0 + hh - 1).min((h - 1) as i64);
      if c0 >= w as i64 || r0 >= h as i64 {
        return;
      }
      hline(grid, glyphs, r0, c0, c1, value, '=');
      hline(grid, glyphs, r1, c0, c1, value, '=');
      vline(grid, glyphs, c0, r0, r1, value, '|');
      vline(grid, glyphs, c1, r0, r1, value, '|');
      let max_n = (c1 - c0 - 1).max(0) as usize;
      for (i, ch) in label.chars().take(max_n).enumerate() {
        put(grid, glyphs, c0 + 1 + i as i64, (r0 + r1) / 2, value + 0.12, ch);
      }
    };
    let top = (h / 6).max(1) as i64;
    let bottom = ((h - 2).min(h - h / 7)) as i64;
    let left = (w / 18).max(1) as i64;
    let right = ((w - 2).min(w - w / 18)) as i64;
    let mid = (w / 2) as i64;
    let rows = [top, (h / 3) as i64, (h / 2) as i64, ((h * 2) / 3) as i64, bottom];
    for (idx, &row) in rows.iter().enumerate() {
      hline(
        &mut grid,
        &mut glyphs,
        row,
        left,
        right,
        0.20 + idx as f64 * 0.025,
        if idx == 2 { '=' } else { '-' },
      );
    }
    let cols = [left, (w / 4) as i64, mid, ((w * 3) / 4) as i64, right];
    for (idx, &col) in cols.iter().enumerate() {
      vline(&mut grid, &mut glyphs, col, top, bottom, 0.18 + idx as f64 * 0.02, '|');
      for &row in &rows {
        put(&mut grid, &mut glyphs, col, row, 0.55, '+');
      }
    }
    let cpu_b =
      ((w / 7).max(2) as i64, (h / 3 - 2).max(2) as i64, (w / 7).max(8) as i64, (h / 4).max(5) as i64);
    let mem_b = (
      ((w / 2).saturating_sub(w / 12)).max(2) as i64,
      (h / 7).max(1) as i64,
      (w / 6).max(10) as i64,
      (h / 6).max(4) as i64,
    );
    let io_b = (
      ((w - (w / 8).max(10) - 2).min(w - w / 5)) as i64,
      (h / 2 - 2).max(2) as i64,
      (w / 8).max(8) as i64,
      (h / 4).max(5) as i64,
    );
    let rf_b = (
      (w / 3).max(2) as i64,
      (((h - (h / 6).max(4) - 1) as i64).min(bottom - 2)).max(0),
      (w / 7).max(9) as i64,
      (h / 6).max(4) as i64,
    );
    bx(&mut grid, &mut glyphs, cpu_b.0, cpu_b.1, cpu_b.2, cpu_b.3, "CPU", 0.50);
    bx(&mut grid, &mut glyphs, mem_b.0, mem_b.1, mem_b.2, mem_b.3, "RAM", 0.44);
    bx(&mut grid, &mut glyphs, io_b.0, io_b.1, io_b.2, io_b.3, "IO", 0.48);
    bx(&mut grid, &mut glyphs, rf_b.0, rf_b.1, rf_b.2, rf_b.3, "ADC", 0.42);
    // Each entry: ((from_xy, to_xy), speed, phase, glyph). Spelling the tuple
    // out keeps the layout legible inline; an alias would just bounce the
    // reader to another file.
    #[allow(clippy::type_complexity)]
    let paths: &[(((i64, i64), (i64, i64)), f64, f64, char)] = &[
      (((left, rows[0]), (right, rows[0])), 3.8, 0.00, '>'),
      (((right, rows[4]), (left, rows[4])), 4.4, 0.19, '<'),
      (((mid, top), (mid, bottom)), 3.2, 0.35, 'v'),
      (((cpu_b.0 + cpu_b.2, cpu_b.1 + 1), (io_b.0, io_b.1 + 1)), 2.7, 0.48, '*'),
      (((mem_b.0 + mem_b.2 / 2, mem_b.1 + mem_b.3), (rf_b.0 + rf_b.2 / 2, rf_b.1)), 3.5, 0.66, '#'),
      (((left, rows[2]), (right, rows[2])), 2.4, 0.82, '@'),
    ];
    for (path_idx, &(((x0, y0), (x1, y1)), speed, offset, head)) in paths.iter().enumerate() {
      let pulse = ((ctx.elapsed / speed + offset) % 1.0 + 1.0) % 1.0;
      for tail in 0..10 {
        let f = pulse - tail as f64 * 0.018;
        let f = if f < 0.0 { f + 1.0 } else { f };
        let col = (x0 as f64 + (x1 - x0) as f64 * f).round() as i64;
        let row = (y0 as f64 + (y1 - y0) as f64 * f).round() as i64;
        let value = 1.0 - tail as f64 * 0.075;
        if value > 0.25 {
          put(&mut grid, &mut glyphs, col, row, value, if tail == 0 { head } else { '*' });
        }
      }
      if path_idx % 2 == 0 {
        let branch_col = (x0 as f64 + (x1 - x0) as f64 * pulse).round() as i64;
        let branch_row = if path_idx == 0 { rows[1] } else { rows[3] };
        vline(&mut grid, &mut glyphs, branch_col, branch_row.min(y0), branch_row.max(y0), 0.30, '|');
        put(&mut grid, &mut glyphs, branch_col, branch_row, 0.85, '@');
      }
    }
    let shimmer = 0.025 * (0.5 + 0.5 * (ctx.elapsed * 2.7).sin());
    let contrast = ctx.options.contrast;
    for v in grid.iter_mut() {
      if *v > 0.0 && *v < 0.55 {
        *v += shimmer;
      }
      *v = clamp(*v * contrast);
    }
    render_glyph_field(ctx, &grid, &glyphs, &STYLE, out);
  }
}
