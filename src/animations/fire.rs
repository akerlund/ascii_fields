use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, render_field, FieldStyle};
use crate::noise::fbm;

const STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "fire" };
const TH: &[(f64, char)] = &[
  (0.10, ' '),
  (0.20, '.'),
  (0.32, ':'),
  (0.44, '-'),
  (0.56, '='),
  (0.68, '+'),
  (0.80, '*'),
  (0.90, '#'),
  (1.01, '@'),
];

pub struct Fire;

impl Animation for Fire {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let w = ctx.width;
    let h = ctx.height;
    let contrast = ctx.options.contrast;
    let mut grid = vec![0.0_f64; w * h];
    let dw = (w.saturating_sub(1)).max(1) as f64;
    let dh = (h.saturating_sub(1)).max(1) as f64;
    for row in 0..h {
      let v = row as f64 / dh;
      let rise = 1.0 - v;
      let base = row * w;
      for col in 0..w {
        let u = col as f64 / dw;
        let sway = 0.10 * (v * 4.0 - ctx.elapsed * 1.5).sin();
        let spread = 0.34 + 0.18 * rise;
        let horiz = (-((u - 0.5 - sway).powi(2)) / (spread * spread)).exp();
        let turb = fbm(u * 5.0, v * 6.0 - ctx.elapsed * 4.2, 4);
        let mut flame = rise.powf(0.6) * horiz * (0.35 + 1.15 * turb);
        flame -= 0.55 * v;
        let flicker = 0.85 + 0.15 * (ctx.elapsed * 9.0 + col as f64 * 0.5).sin();
        grid[base + col] = clamp(flame * flicker * 1.7 * contrast);
      }
    }
    render_field(ctx, &grid, TH, &STYLE, out);
  }
}
