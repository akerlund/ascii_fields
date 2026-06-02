use rayon::prelude::*;

use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, render_field, FieldStyle};
use crate::noise::fbm;

const STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "ice" };
const TH: &[(f64, char)] = &[
  (0.18, ' '),
  (0.30, '.'),
  (0.42, ':'),
  (0.52, '-'),
  (0.62, '='),
  (0.72, '+'),
  (0.82, '*'),
  (0.92, '#'),
  (1.01, '@'),
];

pub struct Clouds;

impl Animation for Clouds {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let w = ctx.width;
    let h = ctx.height;
    let drift = ctx.elapsed * 0.06;
    let freq = 3.2 * ctx.options.scale.max(0.4);
    let contrast = ctx.options.contrast;
    let mut grid = vec![0.0_f64; w * h];
    let dw = (w.saturating_sub(1)).max(1) as f64;
    let dh = (h.saturating_sub(1)).max(1) as f64;
    grid.par_chunks_mut(w).enumerate().for_each(|(row, row_slice)| {
      let v = row as f64 / dh;
      let glow = 0.12 + 0.18 * v;
      let vfreq = v * freq;
      let vfreq14 = v * freq * 1.4;
      let vfreq_drift = vfreq - drift * 0.6;
      for (col, slot) in row_slice.iter_mut().enumerate() {
        let u = col as f64 / dw;
        let ufreq = u * freq;
        let wx = fbm(ufreq + drift, vfreq, 3);
        let wy = fbm(ufreq + 5.2, vfreq_drift, 3);
        let n = fbm(ufreq + drift + wx * 1.5, vfreq14 + wy * 1.5, 5);
        let cloud = ((n - 0.42) / 0.58).max(0.0);
        let level = glow + cloud * 0.95;
        *slot = clamp(level * contrast);
      }
    });
    render_field(ctx, &grid, TH, &STYLE, out);
  }
}
