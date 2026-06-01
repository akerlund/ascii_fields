use std::f64::consts::PI;
use rayon::prelude::*;

use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, render_field, FieldStyle};
use crate::noise::fbm;

const STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "aurora" };
const TH: &[(f64, char)] = &[
  (0.08, ' '), (0.18, '.'), (0.30, ':'), (0.42, '-'), (0.55, '='),
  (0.68, '+'), (0.80, '*'), (0.90, '#'), (1.01, '@'),
];
// (centre, thickness, frequency, drift speed, weight)
const CURTAINS: &[(f64, f64, f64, f64, f64)] = &[
  (0.42, 0.14, 2.3, 0.55, 1.00),
  (0.52, 0.10, 3.7, -0.40, 0.75),
  (0.34, 0.08, 5.1, 0.70, 0.55),
];

pub struct Aurora;

impl Animation for Aurora {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let w = ctx.width; let h = ctx.height;
    let contrast = ctx.options.contrast;
    let mut grid = vec![0.0_f64; w * h];
    let dw = (w.saturating_sub(1)).max(1) as f64;
    let dh = (h.saturating_sub(1)).max(1) as f64;
    let t = ctx.elapsed;
    grid.par_chunks_mut(w).enumerate().for_each(|(row, row_slice)| {
      let v = row as f64 / dh;
      for col in 0..w {
        let u = col as f64 / dw;
        let mut value = 0.04;
        for &(centre, thick, freq, speed, weight) in CURTAINS {
          let cy = centre
            + 0.10 * (freq * u * PI + t * speed).sin()
            + 0.05 * fbm(u * 3.0 + t * 0.1, centre * 4.0, 3);
          let band = (-((v - cy).powi(2)) / (thick * thick)).exp();
          let rays = 0.55 + 0.45 * (u * 60.0 + 8.0 * fbm(u * 6.0, t * 0.3, 4)).sin();
          let fade = (1.0 - v * 0.4).clamp(0.0, 1.0);
          value += weight * band * rays * fade;
        }
        row_slice[col] = clamp(value * contrast);
      }
    });
    render_field(ctx, &grid, TH, &STYLE, out);
  }
}
