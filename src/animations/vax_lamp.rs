use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, render_field, FieldStyle};

const STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "lava" };
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
// (bx, offset, rx, ry, period)
const BLOBS: &[(f64, f64, f64, f64, f64)] = &[
  (0.42, 0.00, 0.17, 0.20, 8.5),
  (0.58, 0.22, 0.13, 0.16, 6.8),
  (0.48, 0.48, 0.20, 0.24, 10.0),
  (0.62, 0.70, 0.11, 0.15, 5.7),
];

struct BlobState {
  cx0: f64,
  sway: f64,
  cy: f64,
  inv_rx: f64,
  inv_ry: f64,
  offset: f64,
}

pub struct VaxLamp;

impl Animation for VaxLamp {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let w = ctx.width;
    let h = ctx.height;
    let ax = w as f64 / (h as f64 * 2.0).max(1.0);
    let contrast = ctx.options.contrast;
    let blob_state: Vec<BlobState> = BLOBS
      .iter()
      .map(|&(bx, offset, rx, ry, period)| {
        let local = ((ctx.elapsed / period + offset) % 1.0 + 1.0) % 1.0;
        let mut cy = 1.15 - local * 2.30;
        if cy < -1.15 {
          cy += 2.30;
        }
        let cx0 = (bx - 0.5) * 2.0 * ax;
        let sway = 0.08 * ax * (ctx.elapsed * 0.55 + offset * 12.0).sin();
        let rise_fraction = (1.0 - (2.0 * local - 1.0).abs()).clamp(0.0, 1.0);
        let stretch = 1.0 + 0.45 * rise_fraction;
        BlobState {
          cx0,
          sway,
          cy,
          inv_rx: 1.0 / (rx * ax).max(0.001),
          inv_ry: 1.0 / (ry * stretch).max(0.001),
          offset,
        }
      })
      .collect();
    let mut grid = vec![0.0_f64; w * h];
    let dw = (w.saturating_sub(1)).max(1) as f64;
    let dh = (h.saturating_sub(1)).max(1) as f64;
    for row in 0..h {
      let v = row as f64 / dh;
      let y = (v - 0.5) * 2.0;
      let lamp_width = 0.34 * ax * (0.72 + 0.25 * (1.0 - y * y));
      let inv_lamp = 1.0 / lamp_width.max(0.001);
      let cap = (-((v - 0.04).powi(2)) / 0.002).exp() + (-((v - 0.96).powi(2)) / 0.002).exp();
      let base = row * w;
      for col in 0..w {
        let u = col as f64 / dw;
        let x = (u - 0.5) * 2.0 * ax;
        let wall = x.abs() * inv_lamp;
        let glass = 0.20 * (-((wall - 1.0).powi(2)) / 0.012).exp();
        let mut value = glass;
        if wall < 1.0 {
          value += 0.05 + 0.12 * (1.0 - v);
          for b in &blob_state {
            let cx = b.cx0 + b.sway * (y * 3.0 + b.offset).sin();
            let dx = (x - cx) * b.inv_rx;
            let dy = (y - b.cy) * b.inv_ry;
            let blob = (-(dx * dx + dy * dy) * 1.5).exp();
            value += blob * 0.78;
          }
          value += cap * 0.26;
        }
        grid[base + col] = clamp(value * contrast);
      }
    }
    render_field(ctx, &grid, TH, &STYLE, out);
  }
}
