use rayon::prelude::*;

use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, render_field, FieldStyle};
use crate::noise::fbm;

const STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "ocean" };
const TH: &[(f64, char)] = &[
  (0.10, ' '), (0.20, '.'), (0.32, ':'), (0.44, '-'), (0.56, '='),
  (0.68, '+'), (0.80, '*'), (0.90, '#'), (1.01, '@'),
];

pub struct Whirlpool;

impl Animation for Whirlpool {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let w = ctx.width; let h = ctx.height;
    let mut grid = vec![0.0_f64; w * h];
    let t = ctx.elapsed;
    let arms = 2.0_f64;
    let windings = 5.0 * ctx.options.scale.max(0.5);
    let contrast = ctx.options.contrast;
    let radius = (w.min(h * 2) as f64 * 0.5).max(1.0);
    let cx_mid = (w as f64 - 1.0) * 0.5;
    let cy_mid = (h as f64 - 1.0) * 0.5;
    grid.par_chunks_mut(w).enumerate().for_each(|(row, row_slice)| {
      let py = ((row as f64 - cy_mid) * 2.0) / radius;
      for col in 0..w {
        let px = (col as f64 - cx_mid) / radius;
        let r = (px * px + py * py).sqrt() + 1e-4;
        let theta = py.atan2(px);
        let swirl = theta + windings * (r + 0.05).ln() + t * (0.6 + 1.4 / (1.0 + 8.0 * r));
        let spiral = 0.5 + 0.5 * (arms * swirl).sin();
        let foam = fbm(swirl.cos() * 3.0 + t * 0.2, swirl.sin() * 3.0, 4);
        let surface = 0.45 * spiral + 0.45 * foam;
        let throat = 1.0 - (-(r * r) / 0.02).exp();
        let rim = (-((r - 0.16).powi(2)) / 0.01).exp() * 0.6;
        let edge = (1.0 - (r - 1.0) * 2.5).max(0.0).min(1.0);
        let level = (surface * throat + rim) * edge;
        row_slice[col] = clamp(level * 1.5 * contrast);
      }
    });
    render_field(ctx, &grid, TH, &STYLE, out);
  }
}
