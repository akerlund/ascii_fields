use rayon::prelude::*;

use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, render_field, FieldStyle};
use crate::noise::fbm;

const STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "sunset" };
const TH: &[(f64, char)] = &[
  (0.08, ' '),
  (0.18, '.'),
  (0.30, ':'),
  (0.42, '-'),
  (0.54, '='),
  (0.66, '+'),
  (0.78, '*'),
  (0.90, '#'),
  (1.01, '@'),
];

pub struct Storm;

impl Animation for Storm {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let w = ctx.width;
    let h = ctx.height;
    let ax = w as f64 / (h as f64 * 2.0).max(1.0);
    let cx = 0.08 * ax * (ctx.elapsed * 0.12).sin();
    let cy = 0.03 * (ctx.elapsed * 0.18).sin();
    let spin = ctx.elapsed * 0.55;
    let contrast = ctx.options.contrast;
    let band_drift = ctx.elapsed * 0.06;
    let cos_spin = spin.cos();
    let sin_spin = spin.sin();
    let inv_rx = 1.0 / (0.36 * ax).max(0.22);
    let mut grid = vec![0.0_f64; w * h];
    let dw = (w.saturating_sub(1)).max(1) as f64;
    let dh = (h.saturating_sub(1)).max(1) as f64;
    let t = ctx.elapsed;
    grid.par_chunks_mut(w).enumerate().for_each(|(row, row_slice)| {
      let y = (row as f64 / dh - 0.5) * 2.0;
      let band_base = 0.10 + 0.13 * (y * 18.0 + t * 0.25).sin() + 0.07 * (y * 43.0 - t * 0.15).sin();
      let shear = 0.12 * (y * 9.0 + t * 0.2).sin();
      let dy = (y - cy) / 0.25;
      let abs_dy = dy.abs();
      let streamer_y = (-((abs_dy - 0.75).powi(2)) / 0.10).exp();
      for col in 0..w {
        let x = (col as f64 / dw - 0.5) * 2.0 * ax;
        let wind = fbm(x * 1.8 + band_drift, y * 5.0 + shear, 3);
        let mut value = band_base + 0.18 * wind;
        let dx = (x - cx) * inv_rx;
        let r = (dx * dx + dy * dy).sqrt();
        let theta = dy.atan2(dx);
        let oval = (-(r * r) * 1.35).exp();
        let wall = (-((r - 0.95).powi(2)) / 0.028).exp();
        let eye = (-(r * r) / 0.12).exp();
        let swirl = 0.5 + 0.5 * (theta * 4.0 - spin + r * 8.5).sin();
        let turbulent = fbm(dx * 2.2 + cos_spin * 0.4, dy * 2.2 + sin_spin * 0.4, 3);
        value += oval * (0.25 + 0.30 * swirl + 0.20 * turbulent);
        value += wall * (0.42 + 0.22 * swirl);
        value -= eye * 0.55;
        value += streamer_y * (1.0 - dx.abs() * 0.9).max(0.0) * 0.18;
        row_slice[col] = clamp(value * contrast);
      }
    });
    render_field(ctx, &grid, TH, &STYLE, out);
  }
}
