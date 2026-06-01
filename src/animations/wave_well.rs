use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, render_field, FieldStyle};

const STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "ocean" };
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
// (amplitude, energy, x-phase, y-phase)
const MODES: &[(f64, f64, f64, f64)] = &[(1.00, 1.0, 0.0, 0.0), (0.70, 2.0, 0.6, 1.1), (0.45, 3.0, 1.7, 0.3)];

pub struct WaveWell;

impl Animation for WaveWell {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let w = ctx.width;
    let h = ctx.height;
    let t = ctx.elapsed * 1.3;
    let cx = 0.42 * t.sin();
    let cy = 0.30 * (1.7 * t + 0.5).sin();
    let sigma = 0.30 + 0.10 * (t * 0.8).sin();
    let contrast = ctx.options.contrast;
    let mut grid = vec![0.0_f64; w * h];
    let dw = (w.saturating_sub(1)).max(1) as f64;
    let dh = (h.saturating_sub(1)).max(1) as f64;
    for row in 0..h {
      let y = (row as f64 / dh - 0.5) * 2.0;
      let base = row * w;
      for col in 0..w {
        let x = (col as f64 / dw - 0.5) * 2.0;
        let well = x * x + y * y;
        let mut re = 0.0_f64;
        let mut im = 0.0_f64;
        for &(amp, energy, phx, phy) in MODES {
          let envelope = (-((x - cx).powi(2) + (y - cy).powi(2)) / (sigma * sigma)).exp();
          let ripple = (6.0 * (x * phx.cos() + y * phy.sin()) - energy * t).cos();
          let rippl2 = (6.0 * (x * phx.cos() + y * phy.sin()) - energy * t).sin();
          re += amp * envelope * ripple;
          im += amp * envelope * rippl2;
        }
        let prob = re * re + im * im;
        let wall = 0.06 * (-((well.sqrt() - 0.95).powi(2)) / 0.01).exp();
        grid[base + col] = clamp(prob * 0.5 * contrast + wall);
      }
    }
    render_field(ctx, &grid, TH, &STYLE, out);
  }
}
