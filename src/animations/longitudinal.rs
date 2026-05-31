use std::f64::consts::PI;
use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, render_field, FieldStyle};

const STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "ice" };
const TH: &[(f64, char)] = &[
  (0.12, ' '), (0.22, '.'), (0.34, ':'), (0.46, '-'), (0.58, '='),
  (0.70, '+'), (0.82, '*'), (0.92, '#'), (1.01, '@'),
];

pub struct Longitudinal;

impl Animation for Longitudinal {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let w = ctx.width; let h = ctx.height;
    let freq = 3.0 * ctx.options.scale.max(0.4);
    let omega = 2.2;
    let amp = (0.55 / freq.max(1.0)).max(0.01);
    let k = 2.0 * PI * freq;
    let contrast = ctx.options.contrast;
    let mut grid = vec![0.0_f64; w * h];
    let dw = (w.saturating_sub(1)).max(1) as f64;
    for row in 0..h {
      let base = row * w;
      for col in 0..w {
        let x = col as f64 / dw;
        let density = 0.5 + 0.5 * (k * x - omega * ctx.elapsed).cos();
        grid[base + col] = 0.10 + 0.16 * density * density;
      }
    }
    let cols = (freq * 10.0) as usize;
    let cols = cols.max(12);
    let row_step = if h >= 10 { 2 } else { 1 };
    for ix in 0..cols {
      let x0 = ix as f64 / cols as f64;
      let dx = amp * (k * x0 - omega * ctx.elapsed).sin();
      let sx = x0 + dx;
      let col = (sx * (w as f64 - 1.0)).round() as i64;
      if col >= 0 && (col as usize) < w {
        let c = col as usize;
        let mut row = 0;
        while row < h {
          if grid[row * w + c] < 1.0 { grid[row * w + c] = 1.0; }
          if c + 1 < w && grid[row * w + c + 1] < 0.55 { grid[row * w + c + 1] = 0.55; }
          row += row_step;
        }
      }
    }
    for v in grid.iter_mut() { *v = clamp(*v * contrast); }
    render_field(ctx, &grid, TH, &STYLE, out);
  }
}
