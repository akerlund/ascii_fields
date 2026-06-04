use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, render_field, FieldStyle};

const STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "amber" };
const TH: &[(f64, char)] =
  &[(0.10, ' '), (0.26, '.'), (0.42, ':'), (0.56, '-'), (0.68, '+'), (0.80, '*'), (0.90, '#'), (1.01, '@')];

pub struct Phyllotaxis;

impl Animation for Phyllotaxis {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let w = ctx.width;
    let h = ctx.height;
    let seeds = (650.0 * ctx.options.scale.max(0.5)) as usize;
    let ax = w as f64 / (h as f64 * 2.0).max(1.0);
    let cx = w as f64 * 0.5;
    let cy = h as f64 * 0.5;
    let golden = std::f64::consts::PI * (3.0 - 5.0_f64.sqrt());
    let radius_world = (w as f64 * 0.5 / ax.max(0.1)).min(h as f64) * 0.95;
    let scale = radius_world / (seeds as f64).sqrt();
    let rot = ctx.elapsed * 0.18;
    let pulse = ctx.elapsed * 0.55;
    let contrast = ctx.options.contrast;
    let mut grid = vec![0.0_f64; w * h];

    for n in 1..=seeds {
      let r = (n as f64).sqrt() * scale;
      let a = n as f64 * golden + rot;
      let px = cx + r * a.cos() * ax;
      let py = cy + r * a.sin() * 0.5;
      // Centre seeds are brightest, edge seeds dim with a sinusoidal pulse
      // travelling outward. (1 - sqrt(n/seeds)) gives a smooth inward falloff
      // and the pulse adds movement so the flower visibly breathes.
      let radial = 1.0 - (n as f64 / seeds as f64).sqrt();
      let bright = clamp(0.45 + 0.45 * radial + 0.18 * (pulse - n as f64 * 0.02).sin());

      // Splat each seed as a 3x3 Gaussian footprint so single seeds actually
      // show up as dots instead of single-pixel marks below the rendering
      // threshold.
      let ci = px.round() as i64;
      let ri = py.round() as i64;
      for dy in -1..=1_i64 {
        let row = ri + dy;
        if row < 0 || row >= h as i64 {
          continue;
        }
        for dx in -1..=1_i64 {
          let col = ci + dx;
          if col < 0 || col >= w as i64 {
            continue;
          }
          let dist2 = (dx * dx + dy * dy) as f64;
          let g = (-dist2 / 0.65).exp();
          let idx = row as usize * w + col as usize;
          let v = bright * g;
          if grid[idx] < v {
            grid[idx] = v;
          }
        }
      }
    }
    if (contrast - 1.0).abs() > 0.001 {
      for v in grid.iter_mut() {
        *v = clamp(*v * contrast);
      }
    }
    render_field(ctx, &grid, TH, &STYLE, out);
  }
}
