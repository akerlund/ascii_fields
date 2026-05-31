use std::f64::consts::TAU;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg32;
use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, render_field, FieldStyle, smoothstep};

const STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "ice" };
const TH: &[(f64, char)] = &[
  (0.05, ' '), (0.20, '.'), (0.40, ':'), (0.58, '-'), (0.72, '+'),
  (0.84, '*'), (0.93, '#'), (1.01, '@'),
];
const CYCLE: f64 = 9.0;

pub struct Lightspeed {
  stars: Vec<(f64, f64)>,
  rng: Pcg32,
}
impl Default for Lightspeed { fn default() -> Self { Self { stars: Vec::new(), rng: Pcg32::from_entropy() } } }

impl Animation for Lightspeed {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let w = ctx.width; let h = ctx.height;
    let n = 60.max(w * h / 18);
    if self.stars.len() != n {
      self.stars = (0..n).map(|_| (
        self.rng.gen_range(0.0..TAU),
        self.rng.gen_range(0.0..1.0),
      )).collect();
    }
    let p = (ctx.elapsed % CYCLE) / CYCLE;
    let streak = (smoothstep(0.40, 0.60, p) - smoothstep(0.90, 1.0, p)).clamp(0.0, 1.0);
    let flash = (-((p - 0.605).powi(2)) / 0.0006).exp();
    let shimmer = 0.85 + 0.15 * (ctx.elapsed * 30.0).sin();
    let cx = w as f64 * 0.5; let cy = h as f64 * 0.5;
    let rmax = (cx * cx + cy * cy).sqrt();
    let flow = ctx.elapsed * (0.04 + streak * 1.4);
    let length = 0.8 + streak * rmax * 1.05 * shimmer;
    let head_bright = 0.45 + 0.5 * streak;
    let mut grid = vec![0.0_f64; w * h];
    for &(ang, r0) in &self.stars {
      let ca = ang.cos(); let sa = ang.sin() * 0.5;
      let r = ((r0 + flow) % 1.0) * rmax;
      let x0 = cx + ca * r; let y0 = cy + sa * r;
      let x1 = cx + ca * (r + length); let y1 = cy + sa * (r + length);
      let steps = ((x1 - x0).hypot(y1 - y0) as i64).max(1);
      for s in 0..=steps {
        let f = s as f64 / steps as f64;
        let px = (x0 + (x1 - x0) * f).round() as i64;
        let py = (y0 + (y1 - y0) * f).round() as i64;
        if px >= 0 && (px as usize) < w && py >= 0 && (py as usize) < h {
          let val = head_bright * (0.35 + 0.65 * f);
          let idx = (py as usize) * w + (px as usize);
          if val > grid[idx] { grid[idx] = val; }
        }
      }
    }
    if flash > 0.01 { for v in grid.iter_mut() { *v = clamp(*v + flash); } }
    render_field(ctx, &grid, TH, &STYLE, out);
  }
}
