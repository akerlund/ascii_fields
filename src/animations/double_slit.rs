use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, render_field, FieldStyle};

const STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "ice" };
const TH: &[(f64, char)] = &[
  (0.10, ' '), (0.20, '.'), (0.32, ':'), (0.44, '-'), (0.56, '='),
  (0.68, '+'), (0.80, '*'), (0.90, '#'), (1.01, '@'),
];

pub struct DoubleSlit {
  screen: Vec<f64>,
  h: usize,
  last: f64,
}
impl Default for DoubleSlit { fn default() -> Self { Self { screen: Vec::new(), h: 0, last: 0.0 } } }

impl Animation for DoubleSlit {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let w = ctx.width; let h = ctx.height;
    if self.screen.len() != h || ctx.elapsed < self.last {
      self.screen = vec![0.0_f64; h]; self.h = h;
    }
    let dt = (ctx.elapsed - self.last).max(0.0);
    self.last = ctx.elapsed;
    let barrier_x = 0.32;
    let slit_y1 = 0.40; let slit_y2 = 0.60;
    let k = 42.0_f64;
    let omega = 7.0_f64;
    let ax = w as f64 / (h as f64 * 2.0).max(1.0);
    let detector_x = 0.93;
    let s1x = barrier_x * ax; let s1y = slit_y1;
    let s2x = barrier_x * ax; let s2y = slit_y2;
    let mut peak = 0.0_f64;
    let dh = (h.saturating_sub(1)).max(1) as f64;
    let dw = (w.saturating_sub(1)).max(1) as f64;
    for row in 0..h {
      let v = row as f64 / dh;
      let dx = detector_x * ax;
      let r1 = ((dx - s1x).powi(2) + (v - s1y).powi(2)).sqrt();
      let r2 = ((dx - s2x).powi(2) + (v - s2y).powi(2)).sqrt();
      let amp = (k * r1).sin() + (k * r2).sin();
      let inten = amp * amp;
      let acc = self.screen[row] + inten * dt * 0.18;
      self.screen[row] = acc;
      if acc > peak { peak = acc; }
    }
    let norm = if peak > 1e-6 { 1.0 / peak } else { 0.0 };
    let contrast = ctx.options.contrast;
    let mut grid = vec![0.0_f64; w * h];
    for row in 0..h {
      let v = row as f64 / dh;
      let fringe = self.screen[row] * norm;
      let base = row * w;
      for col in 0..w {
        let u = col as f64 / dw;
        let x = u * ax;
        let level = if u < barrier_x {
          0.30 + 0.30 * (k * x - omega * ctx.elapsed).sin()
        } else if u < barrier_x + 0.012 {
          let near_slit = (v - slit_y1).abs().min((v - slit_y2).abs());
          if near_slit < 0.035 { 0.0 } else { 0.92 }
        } else if u > detector_x {
          0.12 + 0.88 * fringe
        } else {
          let r1 = ((x - s1x).powi(2) + (v - s1y).powi(2)).sqrt();
          let r2 = ((x - s2x).powi(2) + (v - s2y).powi(2)).sqrt();
          let amp = (k * r1 - omega * ctx.elapsed).sin() / (0.4 + r1)
                  + (k * r2 - omega * ctx.elapsed).sin() / (0.4 + r2);
          0.32 + amp * 0.5
        };
        grid[base + col] = clamp(level * contrast);
      }
    }
    render_field(ctx, &grid, TH, &STYLE, out);
  }
}
