use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, render_field, FieldStyle};

const STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "ice" };
const TH: &[(f64, char)] = &[
  (0.10, ' '), (0.20, '.'), (0.32, ':'), (0.44, '-'), (0.56, '='),
  (0.68, '+'), (0.80, '*'), (0.90, '#'), (1.01, '@'),
];
const CYCLE: f64 = 13.0;
const T_INSPIRAL: f64 = 9.5;
const R_MAX: f64 = 0.62;
const R_MIN: f64 = 0.06;

pub struct GravitationalWaves {
  phi: f64,
  cycle_start: f64,
  last: f64,
}

impl Default for GravitationalWaves {
  fn default() -> Self { Self { phi: 0.0, cycle_start: 0.0, last: 0.0 } }
}

impl Animation for GravitationalWaves {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    if ctx.elapsed < self.last { self.phi = 0.0; self.cycle_start = ctx.elapsed; }
    let mut local = ctx.elapsed - self.cycle_start;
    if local > CYCLE { self.cycle_start = ctx.elapsed; self.phi = 0.0; local = 0.0; }
    let dt = (ctx.elapsed - self.last).clamp(0.0, 0.1);
    self.last = ctx.elapsed;
    let merging = local >= T_INSPIRAL;
    let sep = if !merging {
      let frac = local / T_INSPIRAL;
      R_MAX * (1.0 - frac).powf(0.5) + R_MIN
    } else { R_MIN };
    let omega = (0.6 / sep.powf(1.5)).min(7.0);
    self.phi += omega * dt;
    let amp = clamp(0.25 + 0.9 * (R_MAX - sep) / R_MAX);
    let ring_age = local - T_INSPIRAL;
    let flash = if merging { (-(ring_age * ring_age) / 0.05).exp() } else { 0.0 };
    let radius = (ctx.width.min(ctx.height * 2) as f64 * 0.5).max(1.0);
    let bx1 = sep * self.phi.cos(); let by1 = sep * self.phi.sin();
    let bx2 = -bx1; let by2 = -by1;
    let contrast = ctx.options.contrast;
    let mut grid = vec![0.0_f64; ctx.width * ctx.height];
    for row in 0..ctx.height {
      let py = ((row as f64 - (ctx.height as f64 - 1.0) * 0.5) * 2.0) / radius;
      let base = row * ctx.width;
      for col in 0..ctx.width {
        let px = (col as f64 - (ctx.width as f64 - 1.0) * 0.5) / radius;
        let r = (px * px + py * py).sqrt();
        let a = py.atan2(px);
        let phi_ret = self.phi - omega * r * 1.6;
        let strain = amp * (2.0 * a - 2.0 * phi_ret).cos() / (r + 0.30);
        let envelope = (-r * 0.55).exp() * (r * 3.0).min(1.0);
        let mut level = 0.16 + 0.6 * strain * envelope;
        if merging {
          let rr = ring_age * 1.1;
          level += 0.7 * (-((r - rr).powi(2)) / 0.012).exp();
          level += flash * (-r * 2.0).exp();
        }
        if !merging {
          for &(bx, by) in &[(bx1, by1), (bx2, by2)] {
            let dr = ((px - bx).powi(2) + (py - by).powi(2)).sqrt();
            level += 0.8 * (-(dr * dr) / 0.0016).exp();
            level *= 1.0 - 0.9 * (-(dr * dr) / 0.0006).exp();
          }
        } else {
          let dr2 = px * px + py * py;
          level += 0.9 * (-dr2 / 0.004).exp();
          level *= 1.0 - 0.92 * (-dr2 / 0.0016).exp();
        }
        grid[base + col] = clamp(level * contrast);
      }
    }
    render_field(ctx, &grid, TH, &STYLE, out);
  }
}
