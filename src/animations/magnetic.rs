use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, render_field, FieldStyle};

const STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "ice" };
const TH: &[(f64, char)] = &[
  (0.18, ' '), (0.30, '.'), (0.42, ':'), (0.54, '-'), (0.66, '='),
  (0.76, '+'), (0.85, '*'), (0.93, '#'), (1.01, '@'),
];

pub struct Magnetic;

impl Animation for Magnetic {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let w = ctx.width; let h = ctx.height;
    let mut grid = vec![0.0_f64; w * h];
    let t = ctx.elapsed;
    let flow = t * 2.2;
    let lines = 9.0_f64;
    let d = 0.34_f64;
    let ax = w as f64 / ((h as f64) * 2.0).max(1.0);
    let contrast = ctx.options.contrast;
    let dw = (w.saturating_sub(1)).max(1) as f64;
    let dh = (h.saturating_sub(1)).max(1) as f64;
    for row in 0..h {
      let py = (row as f64 / dh - 0.5) * 2.0;
      let py2 = py * py;
      let base = row * w;
      for col in 0..w {
        let px = (col as f64 / dw - 0.5) * 2.0 * ax;
        let psi = py.atan2(px - d) - py.atan2(px + d);
        let field_lines = 0.5 + 0.5 * (lines * psi - flow).sin();
        let rn = ((px - d).powi(2) + py2).sqrt() + 0.18;
        let rs = ((px + d).powi(2) + py2).sqrt() + 0.18;
        let strength = (0.18 + 0.22 / rn + 0.22 / rs).clamp(0.0, 0.95);
        let pole_cut = (-(rn * rn) * 18.0).exp().max((-(rs * rs) * 18.0).exp());
        let level = field_lines * strength * (1.0 - 0.85 * pole_cut);
        grid[base + col] = clamp(level * contrast);
      }
    }
    render_field(ctx, &grid, TH, &STYLE, out);
  }
}
