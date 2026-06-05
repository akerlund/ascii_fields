use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, render_field, FieldStyle};
use crate::noise::fbm;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg32;

const STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "ocean" };
const TH: &[(f64, char)] = &[
  (0.06, ' '),
  (0.20, '.'),
  (0.34, ':'),
  (0.48, '-'),
  (0.62, '='),
  (0.74, '+'),
  (0.84, '*'),
  (0.93, '#'),
  (1.01, '@'),
];
const NOISE_SCALE: f64 = 0.10;
const SPEED: f64 = 11.0;
const TRAIL_DECAY: f64 = 0.90;

pub struct Curl {
  w: usize,
  h: usize,
  particles: Vec<(f64, f64)>,
  trail: Vec<f64>,
  last: f64,
  rng: Pcg32,
}
impl Default for Curl {
  fn default() -> Self {
    Self { w: 0, h: 0, particles: Vec::new(), trail: Vec::new(), last: 0.0, rng: Pcg32::from_entropy() }
  }
}

impl Curl {
  fn seed(&mut self, w: usize, h: usize) {
    self.w = w;
    self.h = h;
    let n = 80.max(w * h / 28);
    self.particles =
      (0..n).map(|_| (self.rng.gen_range(0.0..w as f64), self.rng.gen_range(0.0..h as f64))).collect();
    self.trail = vec![0.0; w * h];
  }
}

impl Animation for Curl {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    if self.particles.is_empty() || ctx.elapsed < self.last {
      // First frame or rewind: fresh seed.
      self.seed(ctx.width, ctx.height);
    } else if ctx.width != self.w || ctx.height != self.h {
      // Resize: rescale particle positions proportionally so they keep
      // their relative location in the field, and rebuild the trail
      // buffer (the previous trail history is lost but the particles
      // keep flowing).
      let sx = ctx.width as f64 / self.w.max(1) as f64;
      let sy = ctx.height as f64 / self.h.max(1) as f64;
      for p in &mut self.particles {
        p.0 *= sx;
        p.1 *= sy;
      }
      self.trail = vec![0.0; ctx.width * ctx.height];
      self.w = ctx.width;
      self.h = ctx.height;
    }
    let dt = (ctx.elapsed - self.last).clamp(0.0, 0.1);
    let dt = if dt == 0.0 { 0.04 } else { dt };
    self.last = ctx.elapsed;
    for v in self.trail.iter_mut() {
      *v *= TRAIL_DECAY;
    }
    let scale = ctx.options.scale.max(0.4);
    let s = NOISE_SCALE * scale;
    let speed = SPEED * scale;
    let t_off = ctx.elapsed * 0.25;
    let mut new = Vec::with_capacity(self.particles.len());
    let w = ctx.width;
    let h = ctx.height;
    for &(x, y) in &self.particles {
      let u = x * s + t_off;
      let v = y * s;
      let eps = 0.04;
      let nx_p = fbm(u + eps, v, 3);
      let nx_m = fbm(u - eps, v, 3);
      let ny_p = fbm(u, v + eps, 3);
      let ny_m = fbm(u, v - eps, 3);
      let vx = (ny_p - ny_m) / (2.0 * eps) * speed;
      let vy = -(nx_p - nx_m) / (2.0 * eps) * speed;
      let mut nx_pos = (x + vx * dt) % w as f64;
      let mut ny_pos = (y + vy * dt) % h as f64;
      if nx_pos < 0.0 {
        nx_pos += w as f64;
      }
      if ny_pos < 0.0 {
        ny_pos += h as f64;
      }
      let ci = nx_pos as usize;
      let ri = ny_pos as usize;
      if ci < w && ri < h {
        let idx = ri * w + ci;
        if self.trail[idx] < 1.0 {
          self.trail[idx] = (self.trail[idx] + 0.5).min(1.0);
        }
      }
      new.push((nx_pos, ny_pos));
    }
    self.particles = new;
    let contrast = ctx.options.contrast;
    let mut grid = vec![0.0_f64; w * h];
    for (slot, &trail) in grid.iter_mut().zip(self.trail.iter()) {
      *slot = clamp(trail * contrast);
    }
    render_field(ctx, &grid, TH, &STYLE, out);
  }
}
