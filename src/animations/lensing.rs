use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, render_field, FieldStyle};
use crate::noise::{fbm, star_noise};

const STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "ice" };
const TH: &[(f64, char)] = &[
  (0.06, ' '), (0.18, '.'), (0.30, ':'), (0.42, '-'), (0.54, '='),
  (0.66, '+'), (0.78, '*'), (0.90, '#'), (1.01, '@'),
];

fn starfield(bx: f64, by: f64, density: f64) -> f64 {
  let ix0 = (bx * density).floor() as i64;
  let iy0 = (by * density).floor() as i64;
  let mut best = 0.0_f64;
  for di in -1..=1 {
    for dj in -1..=1 {
      let ix = ix0 + di; let iy = iy0 + dj;
      if star_noise(ix, iy) <= 0.92 { continue; }
      let sx = (ix as f64 + star_noise(ix + 101, iy + 7)) / density;
      let sy = (iy as f64 + star_noise(ix + 19, iy + 31)) / density;
      let dx = bx - sx; let dy = by - sy;
      let mag = 0.5 + 0.5 * star_noise(ix + 57, iy + 89);
      let v = mag * (-(dx * dx + dy * dy) * density * density * 6.0).exp();
      if v > best { best = v; }
    }
  }
  best
}

pub struct Lensing;

impl Animation for Lensing {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let w = ctx.width; let h = ctx.height;
    let ax = w as f64 / (h as f64 * 2.0).max(1.0);
    let density = 18.0 * ctx.options.scale.max(0.5);
    let t = ctx.elapsed;
    let lx = 0.95 * ax * (t * 0.18).sin();
    let ly = 0.30 * (t * 0.27).sin();
    let re = 0.28_f64; let re2 = re * re;
    let contrast = ctx.options.contrast;
    let mut grid = vec![0.0_f64; w * h];
    let dw = (w.saturating_sub(1)).max(1) as f64;
    let dh = (h.saturating_sub(1)).max(1) as f64;
    for row in 0..h {
      let py = (row as f64 / dh - 0.5) * 2.0;
      let base = row * w;
      for col in 0..w {
        let px = (col as f64 / dw - 0.5) * 2.0 * ax;
        let dxc = px - lx; let dyc = py - ly;
        let r2 = dxc * dxc + dyc * dyc + 1e-4;
        let factor = re2 / r2;
        let bx = px - dxc * factor;
        let by = py - dyc * factor;
        let star = starfield(bx, by, density);
        let ring_r = r2.sqrt();
        let ring = (-((ring_r - re).powi(2)) / 0.0006).exp() * 0.45;
        let haze = 0.04 + 0.05 * fbm(bx * 2.4, by * 2.4, 3);
        let mut level = star * (1.0 + 1.0 * ring) + ring * 0.22 + haze;
        level *= 1.0 - 0.55 * (-r2 / 0.0014).exp();
        grid[base + col] = clamp(level * contrast);
      }
    }
    render_field(ctx, &grid, TH, &STYLE, out);
  }
}
