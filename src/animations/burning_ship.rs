use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, render_field, FieldStyle};
use super::fractal_base::{axes, iteration_cap, mandelbrot_cell};

const STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "fire" };
const TH: &[(f64, char)] = &[
  (0.04, ' '), (0.13, '.'), (0.25, ':'), (0.38, '-'), (0.52, '='),
  (0.66, '+'), (0.78, '*'), (0.90, '#'), (1.01, '@'),
];
const CENTER: (f64, f64) = (-1.7549, -0.0260);

pub struct BurningShip;

impl Animation for BurningShip {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let log_z = 9.0 * 0.5 * (1.0 - (ctx.elapsed * 0.6 * 0.5).cos());
    let scale = 0.9 * (-log_z).exp();
    let max_iter = iteration_cap(ctx.width, ctx.height, 60, 110);
    let (xs, ys) = axes(ctx.width, ctx.height, CENTER.0, CENTER.1, scale);
    let contrast = ctx.options.contrast;
    let mut grid = vec![0.0_f64; ctx.width * ctx.height];
    for (row, &ci) in ys.iter().enumerate() {
      let base = row * ctx.width;
      for (col, &cr) in xs.iter().enumerate() {
        grid[base + col] = clamp(mandelbrot_cell(cr, ci, max_iter, true) * contrast);
      }
    }
    render_field(ctx, &grid, TH, &STYLE, out);
  }
}
