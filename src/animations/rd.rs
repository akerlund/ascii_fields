use rand::{Rng, SeedableRng};
use rand_pcg::Pcg32;
use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, render_field, FieldStyle};

const STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "aurora" };
const TH: &[(f64, char)] = &[
  (0.08, ' '), (0.20, '.'), (0.32, ':'), (0.44, '-'), (0.56, '='),
  (0.68, '+'), (0.80, '*'), (0.90, '#'), (1.01, '@'),
];
const DU: f64 = 0.16; const DV: f64 = 0.08;
const F: f64 = 0.060; const K: f64 = 0.062;
const STEPS_PER_FRAME: i32 = 6;

pub struct Rd {
  w: usize, h: usize,
  u: Vec<f64>, v: Vec<f64>,
  last: f64,
}
impl Default for Rd { fn default() -> Self { Self { w: 0, h: 0, u: Vec::new(), v: Vec::new(), last: 0.0 } } }

impl Rd {
  fn seed(&mut self, w: usize, h: usize) {
    self.w = w; self.h = h;
    let n = w * h;
    self.u = vec![1.0; n]; self.v = vec![0.0; n];
    for &(cx_f, cy_f) in &[(0.5_f64, 0.5_f64), (0.33, 0.5), (0.66, 0.5)] {
      let cx = (cx_f * w as f64) as i64;
      let cy = (cy_f * h as f64) as i64;
      for dy in -3..=3 {
        for dx in -3..=3 {
          if dx * dx + dy * dy <= 9 {
            let x = (cx + dx).rem_euclid(w as i64) as usize;
            let y = (cy + dy).rem_euclid(h as i64) as usize;
            self.u[y * w + x] = 0.50;
            self.v[y * w + x] = 0.25;
          }
        }
      }
    }
    let mut rng = Pcg32::seed_from_u64(1);
    for _ in 0..(20.max(n / 80)) {
      let x = rng.gen_range(0..w);
      let y = rng.gen_range(0..h);
      self.v[y * w + x] = 0.5;
    }
  }
  fn step(&mut self) {
    let w = self.w; let h = self.h;
    let nu = self.u.clone();
    let nv = self.v.clone();
    let (mut new_u, mut new_v) = (nu.clone(), nv.clone());
    for y in 0..h {
      let ym = (y + h - 1) % h; let yp = (y + 1) % h;
      let base = y * w; let bm = ym * w; let bp = yp * w;
      for x in 0..w {
        let xm = (x + w - 1) % w; let xp = (x + 1) % w;
        let u = nu[base + x]; let v = nv[base + x];
        let lap_u = nu[base + xm] + nu[base + xp] + nu[bm + x] + nu[bp + x] - 4.0 * u;
        let lap_v = nv[base + xm] + nv[base + xp] + nv[bm + x] + nv[bp + x] - 4.0 * v;
        let uvv = u * v * v;
        new_u[base + x] = u + DU * lap_u - uvv + F * (1.0 - u);
        new_v[base + x] = v + DV * lap_v + uvv - (F + K) * v;
      }
    }
    self.u = new_u; self.v = new_v;
  }
}

impl Animation for Rd {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    if self.u.is_empty() || ctx.width != self.w || ctx.height != self.h || ctx.elapsed < self.last {
      self.seed(ctx.width, ctx.height);
    }
    self.last = ctx.elapsed;
    let scale = ctx.options.scale.max(0.4);
    let mut steps = (STEPS_PER_FRAME as f64 * scale) as i32;
    if steps < 1 { steps = 1; }
    if ctx.width * ctx.height > 6000 { steps = steps.max(1) / 2; }
    for _ in 0..steps.max(1) { self.step(); }
    let contrast = ctx.options.contrast;
    let mut grid = vec![0.0_f64; ctx.width * ctx.height];
    for i in 0..grid.len() { grid[i] = clamp(self.v[i] * 4.0 * contrast); }
    render_field(ctx, &grid, TH, &STYLE, out);
  }
}
