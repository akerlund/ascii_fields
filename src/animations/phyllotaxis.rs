use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, render_field, FieldStyle};

const STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "amber" };
const TH: &[(f64, char)] = &[
  (0.10, ' '), (0.26, '.'), (0.42, ':'), (0.56, '-'), (0.68, '+'),
  (0.80, '*'), (0.90, '#'), (1.01, '@'),
];

pub struct Phyllotaxis;

impl Animation for Phyllotaxis {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let w = ctx.width; let h = ctx.height;
    let seeds = (450.0 * ctx.options.scale.max(0.5)) as usize;
    let ax = w as f64 / (h as f64 * 2.0).max(1.0);
    let cx = w as f64 * 0.5; let cy = h as f64 * 0.5;
    let golden = std::f64::consts::PI * (3.0 - 5.0_f64.sqrt());
    let radius_world = (w as f64 * 0.5 / ax.max(0.1)).min(h as f64) * 0.95;
    let scale = radius_world / (seeds as f64).sqrt();
    let rot = ctx.elapsed * 0.18;
    let contrast = ctx.options.contrast;
    let mut grid = vec![0.0_f64; w * h];
    for n in 1..=seeds {
      let r = (n as f64).sqrt() * scale;
      let a = n as f64 * golden + rot;
      let px = cx + r * a.cos() * ax;
      let py = cy + r * a.sin() * 0.5;
      let bright = clamp(0.55 + 0.45 * (ctx.elapsed * 0.55 + n as f64 * 0.06).sin());
      let ci = px.round() as i64; let ri = py.round() as i64;
      if ci >= 0 && (ci as usize) < w && ri >= 0 && (ri as usize) < h {
        let idx = (ri as usize) * w + (ci as usize);
        if grid[idx] < bright { grid[idx] = bright; }
        let idx2 = (ri as usize) * w + (ci as usize + 1).min(w - 1);
        if grid[idx2] < bright * 0.6 { grid[idx2] = bright * 0.6; }
      }
    }
    if (contrast - 1.0).abs() > 0.001 {
      for v in grid.iter_mut() { *v = clamp(*v * contrast); }
    }
    render_field(ctx, &grid, TH, &STYLE, out);
  }
}
