use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, render_field, FieldStyle};
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg32;

const STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "aurora" };
const TH: &[(f64, char)] = &[
  (0.08, ' '),
  (0.20, '.'),
  (0.32, ':'),
  (0.44, '-'),
  (0.56, '='),
  (0.68, '+'),
  (0.80, '*'),
  (0.90, '#'),
  (1.01, '@'),
];
const DU: f64 = 0.16;
const DV: f64 = 0.08;
const F: f64 = 0.060;
const K: f64 = 0.062;
const STEPS_PER_FRAME: i32 = 6;

pub struct Rd {
  w: usize,
  h: usize,
  u: Vec<f64>,
  v: Vec<f64>,
  // Scratch buffers reused every step instead of allocating fresh ones.
  // Each Vec is w*h * 8 bytes; without these we'd allocate ~128 KB per step,
  // ~768 KB of garbage per frame at 6 steps/frame.
  u_buf: Vec<f64>,
  v_buf: Vec<f64>,
  last: f64,
}
impl Default for Rd {
  fn default() -> Self {
    Self { w: 0, h: 0, u: Vec::new(), v: Vec::new(), u_buf: Vec::new(), v_buf: Vec::new(), last: 0.0 }
  }
}

impl Rd {
  fn seed(&mut self, w: usize, h: usize) {
    self.w = w;
    self.h = h;
    let n = w * h;
    self.u = vec![1.0; n];
    self.v = vec![0.0; n];
    self.u_buf = vec![0.0; n];
    self.v_buf = vec![0.0; n];
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
    let w = self.w;
    let h = self.h;
    // Read from u/v, write into u_buf/v_buf, then swap. Zero allocations.
    for y in 0..h {
      let ym = (y + h - 1) % h;
      let yp = (y + 1) % h;
      let base = y * w;
      let bm = ym * w;
      let bp = yp * w;
      for x in 0..w {
        let xm = (x + w - 1) % w;
        let xp = (x + 1) % w;
        let u = self.u[base + x];
        let v = self.v[base + x];
        let lap_u = self.u[base + xm] + self.u[base + xp] + self.u[bm + x] + self.u[bp + x] - 4.0 * u;
        let lap_v = self.v[base + xm] + self.v[base + xp] + self.v[bm + x] + self.v[bp + x] - 4.0 * v;
        let uvv = u * v * v;
        self.u_buf[base + x] = u + DU * lap_u - uvv + F * (1.0 - u);
        self.v_buf[base + x] = v + DV * lap_v + uvv - (F + K) * v;
      }
    }
    std::mem::swap(&mut self.u, &mut self.u_buf);
    std::mem::swap(&mut self.v, &mut self.v_buf);
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
    if steps < 1 {
      steps = 1;
    }
    if ctx.width * ctx.height > 6000 {
      steps = steps.max(1) / 2;
    }
    for _ in 0..steps.max(1) {
      self.step();
    }
    let contrast = ctx.options.contrast;
    let mut grid = vec![0.0_f64; ctx.width * ctx.height];
    for (slot, &v) in grid.iter_mut().zip(self.v.iter()) {
      *slot = clamp(v * 4.0 * contrast);
    }
    render_field(ctx, &grid, TH, &STYLE, out);
  }
}
