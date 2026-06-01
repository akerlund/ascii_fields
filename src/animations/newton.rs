use super::fractal_base::{axes, iteration_cap};
use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, render_field, FieldStyle};
use std::f64::consts::PI;

const STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "spectrum" };
const TH: &[(f64, char)] = &[
  (0.05, ' '),
  (0.16, '.'),
  (0.28, ':'),
  (0.40, '-'),
  (0.52, '='),
  (0.64, '+'),
  (0.76, '*'),
  (0.88, '#'),
  (1.01, '@'),
];

#[inline]
fn cmul(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
  (a.0 * b.0 - a.1 * b.1, a.0 * b.1 + a.1 * b.0)
}
#[inline]
fn csub(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
  (a.0 - b.0, a.1 - b.1)
}
#[inline]
fn cdiv(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
  let d = b.0 * b.0 + b.1 * b.1 + 1e-9;
  ((a.0 * b.0 + a.1 * b.1) / d, (a.1 * b.0 - a.0 * b.1) / d)
}

pub struct Newton;

impl Animation for Newton {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let angle = ctx.elapsed * 0.25;
    let log_z = 1.6 * 0.5 * (1.0 - (ctx.elapsed * 0.4 * 0.5).cos());
    let scale = 1.7 * (-log_z).exp();
    let max_iter = (iteration_cap(ctx.width, ctx.height, 18, 40)).min(40);
    let target = ((3.0 * angle).cos(), (3.0 * angle).sin());
    let roots: [(f64, f64); 3] = [0, 1, 2].map(|k| {
      let a = angle + (k as f64) * 2.0 * PI / 3.0;
      (a.cos(), a.sin())
    });
    let (xs, ys) = axes(ctx.width, ctx.height, 0.0, 0.0, scale);
    let contrast = ctx.options.contrast;
    let mut grid = vec![0.0_f64; ctx.width * ctx.height];
    for (row, &y0) in ys.iter().enumerate() {
      let base = row * ctx.width;
      for (col, &x0) in xs.iter().enumerate() {
        let mut z = (x0, y0);
        for _ in 0..max_iter {
          let z2 = cmul(z, z);
          let z3 = cmul(z2, z);
          let num = csub(z3, target);
          let denom = (3.0 * z2.0 + 1e-9, 3.0 * z2.1);
          z = csub(z, cdiv(num, denom));
        }
        let mut level = 0.0_f64;
        for (k, &root) in roots.iter().enumerate() {
          let dr = csub(z, root);
          let mag = (dr.0 * dr.0 + dr.1 * dr.1).sqrt();
          if mag < 0.1 {
            level = 0.30 + 0.30 * k as f64 + 0.30 * (-mag * 6.0).exp();
            break;
          }
        }
        grid[base + col] = clamp(level * contrast);
      }
    }
    render_field(ctx, &grid, TH, &STYLE, out);
  }
}
