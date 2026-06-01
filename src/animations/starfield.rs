use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, render_field, FieldStyle};
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg32;

const STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "ice" };
const TH: &[(f64, char)] =
  &[(0.05, ' '), (0.20, '.'), (0.40, ':'), (0.58, '+'), (0.74, '*'), (0.86, 'o'), (0.94, '#'), (1.01, '@')];
const STAR_DENSITY: f64 = 0.05;

pub struct Starfield {
  stars: Vec<(f64, f64, f64)>, // x, y, z
  w: usize,
  h: usize,
  last: f64,
  rng: Pcg32,
}
impl Default for Starfield {
  fn default() -> Self {
    Self { stars: Vec::new(), w: 0, h: 0, last: 0.0, rng: Pcg32::from_entropy() }
  }
}

impl Starfield {
  fn seed(&mut self, w: usize, h: usize) {
    self.w = w;
    self.h = h;
    let count = 40.max((w as f64 * h as f64 * STAR_DENSITY) as usize);
    self.stars = (0..count)
      .map(|_| (self.rng.gen_range(-1.0..1.0), self.rng.gen_range(-1.0..1.0), self.rng.gen_range(0.1..1.0)))
      .collect();
  }
}

impl Animation for Starfield {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    if self.stars.is_empty() || ctx.width != self.w || ctx.height != self.h || ctx.elapsed < self.last {
      self.seed(ctx.width, ctx.height);
    }
    let dt = (ctx.elapsed - self.last).clamp(0.0, 0.1);
    self.last = ctx.elapsed;
    let speed = 0.55 * ctx.options.scale.max(0.4);
    let mut grid = vec![0.0_f64; ctx.width * ctx.height];
    let aspect = ctx.height as f64 * 2.0 / ctx.width.max(1) as f64;
    let cx = ctx.width as f64 * 0.5;
    let cy = ctx.height as f64 * 0.5;
    let fov = 0.9_f64;
    for star in self.stars.iter_mut() {
      star.2 -= speed * dt;
      if star.2 <= 0.02 {
        star.0 = self.rng.gen_range(-1.0..1.0);
        star.1 = self.rng.gen_range(-1.0..1.0);
        star.2 = 1.0;
      }
      let z = star.2;
      let sx = cx + (star.0 / z) * fov * cx;
      let sy = cy + (star.1 / z) * fov * cy * aspect;
      let bright = clamp((1.0 - z).powf(1.5) + 0.15);
      let tz = (z + speed * 0.06).min(1.0);
      let tx = cx + (star.0 / tz) * fov * cx;
      let ty = cy + (star.1 / tz) * fov * cy * aspect;
      let steps = ((sx - tx).hypot(sy - ty) as i64).max(1);
      for s in 0..=steps {
        let f = s as f64 / steps as f64;
        let px = (tx + (sx - tx) * f).round() as i64;
        let py = (ty + (sy - ty) * f).round() as i64;
        if px >= 0 && (px as usize) < ctx.width && py >= 0 && (py as usize) < ctx.height {
          let val = bright * (0.4 + 0.6 * f);
          let idx = (py as usize) * ctx.width + (px as usize);
          if val > grid[idx] {
            grid[idx] = val;
          }
        }
      }
    }
    render_field(ctx, &grid, TH, &STYLE, out);
  }
}
