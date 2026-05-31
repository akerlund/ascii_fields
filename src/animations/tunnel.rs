use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, render_field, FieldStyle};
use crate::noise::fbm;

const STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "nebula" };
const TH: &[(f64, char)] = &[
  (0.10, ' '), (0.20, '.'), (0.32, ':'), (0.44, '-'), (0.56, '='),
  (0.68, '+'), (0.80, '*'), (0.90, '#'), (1.01, '@'),
];

pub struct Tunnel;

impl Animation for Tunnel {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let w = ctx.width; let h = ctx.height;
    let t = ctx.elapsed;
    let ax = w as f64 / (h as f64 * 2.0).max(1.0);
    let cx = 0.5 + 0.22 * (t * 0.6).sin();
    let cy = 0.5 + 0.18 * (t * 0.8).cos();
    let rings = 7.0 * ctx.options.scale.max(0.4);
    let contrast = ctx.options.contrast;
    let mut grid = vec![0.0_f64; w * h];
    let dw = (w.saturating_sub(1)).max(1) as f64;
    let dh = (h.saturating_sub(1)).max(1) as f64;
    for row in 0..h {
      let v = row as f64 / dh;
      let dy = v - cy;
      let base = row * w;
      for col in 0..w {
        let u = col as f64 / dw;
        let dx = (u - cx) * ax;
        let r = (dx * dx + dy * dy).sqrt() + 1e-4;
        let angle = dy.atan2(dx);
        let depth = 1.0 / r + t * 1.5;
        let ring = 0.5 + 0.5 * (depth * rings).sin();
        let stripe = 0.5 + 0.5 * (angle * 8.0 + depth * 0.5).sin();
        let grime = fbm(angle * 2.0, depth * 0.4, 3);
        let wall = 0.35 * ring + 0.35 * stripe + 0.30 * grime;
        let lighting = (r * 2.2).clamp(0.0, 1.0);
        grid[base + col] = clamp(wall * lighting * 1.4 * contrast);
      }
    }
    render_field(ctx, &grid, TH, &STYLE, out);
  }
}
