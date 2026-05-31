use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, render_field, FieldStyle};
use crate::noise::fbm;

const STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "ice" };
const TH: &[(f64, char)] = &[
  (0.18, ' '), (0.30, '.'), (0.42, ':'), (0.52, '-'), (0.62, '='),
  (0.72, '+'), (0.82, '*'), (0.92, '#'), (1.01, '@'),
];

pub struct Clouds;

impl Animation for Clouds {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let w = ctx.width; let h = ctx.height;
    let drift = ctx.elapsed * 0.06;
    let freq = 3.2 * ctx.options.scale.max(0.4);
    let contrast = ctx.options.contrast;
    let mut grid = vec![0.0_f64; w * h];
    let dw = (w.saturating_sub(1)).max(1) as f64;
    let dh = (h.saturating_sub(1)).max(1) as f64;
    for row in 0..h {
      let v = row as f64 / dh;
      let glow = 0.12 + 0.18 * v;
      let base = row * w;
      for col in 0..w {
        let u = col as f64 / dw;
        let wx = fbm(u * freq + drift, v * freq, 3);
        let wy = fbm(u * freq + 5.2, v * freq - drift * 0.6, 3);
        let n = fbm(u * freq + drift + wx * 1.5, v * freq * 1.4 + wy * 1.5, 5);
        let cloud = ((n - 0.42) / 0.58).max(0.0);
        let level = glow + cloud * 0.95;
        grid[base + col] = clamp(level * contrast);
      }
    }
    render_field(ctx, &grid, TH, &STYLE, out);
  }
}
