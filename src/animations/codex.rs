//! FlowerSphere -- a dark sphere with a rotating flower-like ASCII texture.

use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, render_field, FieldStyle};
use std::f64::consts::PI;

const STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "rose" };
const RAMP: &[char] =
  &[' ', ' ', '.', '.', ':', ':', '-', '-', '=', '=', '+', '+', '*', '*', '#', '#', '%', '%'];

fn thresholds() -> [(f64, char); 18] {
  let mut out = [(0.0_f64, ' '); 18];
  let n = RAMP.len();
  for (i, ch) in RAMP.iter().enumerate() {
    out[i] = ((((i + 1) as f64) / n as f64).min(1.01), *ch);
  }
  out
}

pub struct FlowerSphere;

fn texture(u: f64, v: f64, phase: f64, scale: f64) -> f64 {
  let x = 2.0 * PI * u;
  let y = 2.0 * PI * v;
  let t = 2.0 * PI * phase;
  let density = scale.round().max(1.0);
  let vertical = ((5.0 * density) * x + 0.85 * (3.0 * y + t).sin()).sin().abs();
  let diagonal = ((3.0 * density) * x - (2.0 * density) * y - t).sin().abs();
  let fine = ((11.0 * density) * x + (4.0 * density) * y + 2.0 * t).sin().abs();
  let interference = ((2.0 * density) * x + (5.0 * y - t).sin()).sin().abs();
  0.38 * vertical + 0.25 * diagonal + 0.22 * fine + 0.15 * interference
}

impl Animation for FlowerSphere {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let w = ctx.width;
    let h = ctx.height;
    let mut grid = vec![0.0_f64; w * h];
    let radius = (w.min(h * 2) as f64 * 0.5).max(1.0);
    let contrast = ctx.options.contrast;
    for row in 0..h {
      let py = ((row as f64 - (h as f64 - 1.0) * 0.5) * 2.0) / radius;
      let base = row * w;
      for col in 0..w {
        let px = (col as f64 - (w as f64 - 1.0) * 0.5) / radius;
        let r = (px * px + py * py).sqrt();
        if r >= 1.0 {
          continue;
        }
        let u = (py.atan2(px) / (2.0 * PI) + 0.5 + 0.025 * (2.0 * PI * ctx.phase).sin()).rem_euclid(1.0);
        let v = py.clamp(-1.0, 1.0).asin() / PI + 0.5;
        let texv = texture(u, v, ctx.phase, ctx.options.scale);
        let edge_fade = (1.0 - r.powf(2.8)).max(0.0);
        let center_dip = 1.0 - 0.18 * (-7.0 * r * r).exp();
        grid[base + col] = clamp(texv * edge_fade * center_dip * contrast);
      }
    }
    let th = thresholds();
    render_field(ctx, &grid, &th, &STYLE, out);
  }
}
