use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, render_field, FieldStyle};
use super::fractal_base::{axes, iteration_cap, julia_cell};

const STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "plasma" };
const TH: &[(f64, char)] = &[
  (0.04, ' '), (0.13, '.'), (0.25, ':'), (0.38, '-'), (0.52, '='),
  (0.66, '+'), (0.78, '*'), (0.90, '#'), (1.01, '@'),
];

pub struct Julia;

impl Animation for Julia {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let a = ctx.elapsed * 0.13;
    let radius = 0.7885_f64;
    let cr = radius * a.cos(); let ci = radius * a.sin();
    let log_z = 2.2 * 0.5 * (1.0 - (ctx.elapsed * 0.5 * 0.6).cos());
    let scale = 1.6 * (-log_z).exp();
    let max_iter = iteration_cap(ctx.width, ctx.height, 60, 110);
    let (xs, ys) = axes(ctx.width, ctx.height, 0.0, 0.0, scale);
    let contrast = ctx.options.contrast;
    let mut grid = vec![0.0_f64; ctx.width * ctx.height];
    for (row, &y0) in ys.iter().enumerate() {
      let base = row * ctx.width;
      for (col, &x0) in xs.iter().enumerate() {
        grid[base + col] = clamp(julia_cell(x0, y0, cr, ci, max_iter) * contrast);
      }
    }
    render_field(ctx, &grid, TH, &STYLE, out);
  }
}
