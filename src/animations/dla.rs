use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, render_field, FieldStyle};
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg32;

const STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "ice" };
const TH: &[(f64, char)] = &[
  (0.05, ' '),
  (0.20, '.'),
  (0.34, ':'),
  (0.48, '-'),
  (0.62, '='),
  (0.74, '+'),
  (0.85, '*'),
  (0.93, '#'),
  (1.01, '@'),
];
const MAX_WALK: i32 = 1500;

pub struct Dla {
  w: usize,
  h: usize,
  cluster: Vec<bool>,
  age: Vec<i32>,
  gen: i32,
  last: f64,
  rng: Pcg32,
}
impl Default for Dla {
  fn default() -> Self {
    Self { w: 0, h: 0, cluster: Vec::new(), age: Vec::new(), gen: 0, last: 0.0, rng: Pcg32::from_entropy() }
  }
}

impl Dla {
  fn seed(&mut self, w: usize, h: usize) {
    self.w = w;
    self.h = h;
    self.cluster = vec![false; w * h];
    self.age = vec![0; w * h];
    let cx = w / 2;
    let cy = h / 2;
    self.cluster[cy * w + cx] = true;
    self.age[cy * w + cx] = 1;
    self.gen = 1;
  }
  fn walk_one(&mut self) {
    let w = self.w;
    let h = self.h;
    let mut x = 0i64;
    let mut y = 0i64;
    for _ in 0..20 {
      x = self.rng.gen_range(0..w as i64);
      y = self.rng.gen_range(0..h as i64);
      if !self.cluster[(y as usize) * w + (x as usize)] {
        break;
      }
    }
    for _ in 0..MAX_WALK {
      let base = (y as usize) * w;
      for &(dx, dy) in &[(1i64, 0i64), (-1, 0), (0, 1), (0, -1)] {
        let nx = (x + dx).rem_euclid(w as i64);
        let ny = (y + dy).rem_euclid(h as i64);
        if self.cluster[(ny as usize) * w + (nx as usize)] {
          self.cluster[base + (x as usize)] = true;
          self.gen += 1;
          self.age[base + (x as usize)] = self.gen;
          return;
        }
      }
      let &(dx, dy) = &[(1i64, 0i64), (-1, 0), (0, 1), (0, -1)][self.rng.gen_range(0..4)];
      x = (x + dx).rem_euclid(w as i64);
      y = (y + dy).rem_euclid(h as i64);
    }
  }
}

impl Animation for Dla {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    if self.cluster.is_empty() || ctx.width != self.w || ctx.height != self.h || ctx.elapsed < self.last {
      self.seed(ctx.width, ctx.height);
    }
    self.last = ctx.elapsed;
    let scale = ctx.options.scale.max(1.0) as usize;
    let walkers = (40.max(self.w * self.h / 40)) * scale;
    let cap = (self.w as f64 * self.h as f64 * 0.55) as usize;
    let size: usize = self.cluster.iter().filter(|&&b| b).count();
    if size >= cap {
      self.seed(ctx.width, ctx.height);
    } else {
      for _ in 0..walkers {
        self.walk_one();
      }
    }
    let max_age = self.gen.max(1) as f64;
    let contrast = ctx.options.contrast;
    let mut grid = vec![0.0_f64; ctx.width * ctx.height];
    for r in 0..ctx.height {
      let base = r * ctx.width;
      for c in 0..ctx.width {
        let a = self.age[base + c];
        if a > 0 {
          grid[base + c] = clamp((0.22 + 0.78 * a as f64 / max_age) * contrast);
        }
      }
    }
    render_field(ctx, &grid, TH, &STYLE, out);
  }
}
