use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, render_field, smoothstep, FieldStyle};
use std::f64::consts::PI;

const STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "ice" };
const TH: &[(f64, char)] = &[
  (0.12, ' '),
  (0.22, '.'),
  (0.34, ':'),
  (0.46, '-'),
  (0.58, '='),
  (0.70, '+'),
  (0.82, '*'),
  (0.92, '#'),
  (1.01, '@'),
];
const CYCLE: f64 = 8.5;
const WAVE_SPEED: f64 = 0.18;
const LANES: &[f64] = &[-0.28, 0.22];

fn mach_at(p: f64, seed: f64) -> f64 {
  let top = 1.85 + 0.45 * seed;
  0.70 + top * smoothstep(0.04, 0.92, p)
}

pub struct Mach;

impl Animation for Mach {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let w = ctx.width;
    let h = ctx.height;
    let ax = w as f64 / (h as f64 * 2.0).max(1.0);
    let contrast = ctx.options.contrast;
    let mut grid = vec![0.0_f64; w * h];
    let dw = (w.saturating_sub(1)).max(1) as f64;
    let dh = (h.saturating_sub(1)).max(1) as f64;
    for row in 0..h {
      let py = (row as f64 / dh - 0.5) * 2.0;
      let base = row * w;
      for col in 0..w {
        let px = (col as f64 / dw - 0.5) * 2.0 * ax;
        let mut v = 0.0_f64;
        for (idx, &lane_y) in LANES.iter().enumerate() {
          let local = ((ctx.elapsed / CYCLE + idx as f64 * 0.50) % 1.0 + 1.0) % 1.0;
          let seed = (idx + 1) as f64;
          let mach = mach_at(local, seed);
          let sx = (-1.45 + 3.10 * local) * ax;
          let sy = lane_y + 0.035 * (ctx.elapsed * 0.55 + idx as f64 * 2.4).sin();
          let cone_angle = if mach > 1.0 { (1.0 / mach).asin() } else { PI * 0.5 };
          let cone_slope = cone_angle.tan();
          let dist = ((px - sx).powi(2) + (py - sy).powi(2)).sqrt();
          let rings = ((dist - ctx.elapsed * WAVE_SPEED * (1.0 + idx as f64 * 0.1)) * 52.0).sin();
          v += rings.max(0.0) * 0.12 * (-dist * 1.6).exp();
          if mach > 1.0 && px < sx {
            let behind = sx - px;
            let edge = (py - sy).abs() - behind * cone_slope;
            let cone = (-(edge * edge) / 0.0012).exp() * (-behind * 0.28).exp();
            let interior = smoothstep(0.10, 0.0, edge) * 0.14 * (-behind * 0.22).exp();
            v += cone * (1.04 + idx as f64 * 0.15) + interior;
          }
          v += 0.86 * (-((px - sx).powi(2) + (py - sy).powi(2)) / 0.0008).exp();
        }
        grid[base + col] = clamp(v * contrast);
      }
    }
    render_field(ctx, &grid, TH, &STYLE, out);
  }
}
