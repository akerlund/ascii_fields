use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, render_field, smoothstep, FieldStyle};
use crate::noise::star_noise;

const STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "amber" };
const TH: &[(f64, char)] = &[
  (0.10, ' '), (0.18, '.'), (0.28, ':'), (0.39, '-'), (0.51, '='),
  (0.64, '+'), (0.78, '*'), (0.91, '#'), (1.01, '%'),
];

#[inline]
fn band(x: f64, y: f64, radius: f64, thickness: f64, t: f64, density: f64, boost: f64) -> f64 {
  let dr = (x * x + y * y).sqrt();
  let dth = y.atan2(x);
  let ring = (-((dr - radius).powi(2)) / thickness).exp();
  let az = 0.62 + 0.38 * (9.0 * dth - 2.2 * t).sin();
  let turb = 0.72 + 0.28 * (31.0 * dr * density + 5.0 * dth - 3.4 * t).sin().abs();
  ring * az * turb * boost
}

pub struct BlackHole;

impl Animation for BlackHole {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let w = ctx.width; let h = ctx.height;
    let mut grid = vec![0.0_f64; w * h];
    let radius = (w.min(h * 2) as f64 * 0.44).max(1.0);
    let density = ctx.options.scale.max(0.45);
    let t = ctx.elapsed * 0.55;
    let contrast = ctx.options.contrast;
    for row in 0..h {
      let py = ((row as f64 - (h as f64 - 1.0) * 0.5) * 2.0) / radius;
      let base = row * w;
      for col in 0..w {
        let px = (col as f64 - (w as f64 - 1.0) * 0.5) / radius;
        let r = (px * px + py * py).sqrt();
        let lens = 0.18 / (r * r + 0.08);
        let bent_y = py + lens * if py >= 0.0 { 1.0 } else { -1.0 };
        let disk_y = bent_y / 0.18;
        let front_y = (py + 0.12) / 0.16;
        let back_y = (py - 0.18 - 0.50 * (-9.0 * px * px).exp()) / 0.13;
        let front = band(px, front_y, 0.92, 0.035, t, density, 0.95);
        let upper_arc = band(px, back_y, 0.98, 0.050, t * 0.85 + 1.7, density, 0.75);
        let lower_lens = 0.42 * band(px, disk_y, 1.08, 0.060, -t * 0.55, density, 0.55);
        let ring_glow = (-((r - 0.34).powi(2)) / 0.005).exp() * 0.38;
        let photon_ring = (-((r - 0.43).powi(2)) / 0.0018).exp() * 0.55;
        let shadow = smoothstep(0.41, 0.34, r);
        let hole_cut = smoothstep(0.48, 0.39, r);
        let doppler = 0.74 + 0.42 * smoothstep(-0.45, 0.65, px);
        let disk = (front * doppler + upper_arc + lower_lens) * (1.0 - 0.70 * hole_cut);
        let halo = 0.13 * (-2.5 * (r - 0.36).max(0.0)).exp();
        let mut bg_stars = 0.0_f64;
        if star_noise(col as i64 + 101, row as i64 - 31) > 0.996 {
          bg_stars = 0.45 + 0.25 * (ctx.elapsed * 1.7 + col as f64).sin();
        }
        let mut texture = disk + photon_ring + ring_glow + halo + bg_stars;
        texture *= 1.0 - 0.96 * shadow;
        let vignette = (1.0 - 0.08 * px.abs() - 0.12 * py.abs()).max(0.0);
        grid[base + col] = clamp(texture * vignette * contrast);
      }
    }
    render_field(ctx, &grid, TH, &STYLE, out);
  }
}
