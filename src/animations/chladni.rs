use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, render_field, FieldStyle};
use std::f64::consts::PI;

const STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "amber" };
const TH: &[(f64, char)] = &[
  (0.10, ' '),
  (0.22, '.'),
  (0.34, ':'),
  (0.46, '-'),
  (0.58, '='),
  (0.70, '+'),
  (0.82, '*'),
  (0.92, '#'),
  (1.01, '@'),
];
const MODES: &[(f64, f64)] = &[
  (2.0, 3.0),
  (3.0, 4.0),
  (4.0, 5.0),
  (5.0, 6.0),
  (4.0, 7.0),
  (3.0, 5.0),
  (6.0, 7.0),
  (2.0, 5.0),
  (5.0, 3.0),
  (7.0, 4.0),
];
const MODE_SECONDS: f64 = 4.5;
const TRANSITION: f64 = 1.4;

pub struct Chladni;

impl Animation for Chladni {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let n_modes = MODES.len();
    let idx = ((ctx.elapsed / MODE_SECONDS) as usize) % n_modes;
    let nxt = (idx + 1) % n_modes;
    let frac = (ctx.elapsed % MODE_SECONDS) / MODE_SECONDS;
    let blend = clamp((frac - (1.0 - TRANSITION / MODE_SECONDS)) / (TRANSITION / MODE_SECONDS));
    let (m0, n0) = MODES[idx];
    let (m1, n1) = MODES[nxt];
    let omega = 1.6;
    let pulse = 0.78 + 0.22 * (omega * ctx.elapsed).cos().abs();
    let contrast = ctx.options.contrast;
    let w = ctx.width;
    let h = ctx.height;
    let mut grid = vec![0.0_f64; w * h];
    let dw = (w.saturating_sub(1)).max(1) as f64;
    let dh = (h.saturating_sub(1)).max(1) as f64;
    for row in 0..h {
      let y = row as f64 / dh;
      let sin0y_m0 = (m0 * PI * y).sin();
      let sin0y_n0 = (n0 * PI * y).sin();
      let sin1y_m1 = (m1 * PI * y).sin();
      let sin1y_n1 = (n1 * PI * y).sin();
      let base = row * w;
      for col in 0..w {
        let x = col as f64 / dw;
        let u0 = (m0 * PI * x).sin() * sin0y_n0 - (n0 * PI * x).sin() * sin0y_m0;
        let u1 = (m1 * PI * x).sin() * sin1y_n1 - (n1 * PI * x).sin() * sin1y_m1;
        let u = u0 * (1.0 - blend) + u1 * blend;
        let nodal = (-u.abs() * 6.0).exp();
        grid[base + col] = clamp(nodal * pulse * contrast);
      }
    }
    render_field(ctx, &grid, TH, &STYLE, out);
  }
}
