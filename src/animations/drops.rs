use rand::{Rng, SeedableRng};
use rand_pcg::Pcg32;
use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, render_field, FieldStyle};

const STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "ocean" };
const TH: &[(f64, char)] = &[
  (0.30, '.'), (0.40, ':'), (0.47, '-'), (0.53, '='), (0.60, '+'),
  (0.70, '*'), (0.82, '#'), (1.01, '%'),
];
const WAVE_SPEED: f64 = 0.55;
const SPAWN_DT: f64 = 0.55;

#[derive(Clone, Copy)] struct Drop { x: f64, y: f64, t0: f64, strength: f64 }

pub struct Drops {
  drops: Vec<Drop>,
  next_spawn: f64,
  last_elapsed: f64,
  rng: Pcg32,
}
impl Default for Drops {
  fn default() -> Self {
    Self { drops: Vec::new(), next_spawn: 0.0, last_elapsed: 0.0, rng: Pcg32::from_entropy() }
  }
}

impl Animation for Drops {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    if ctx.elapsed < self.last_elapsed { self.drops.clear(); self.next_spawn = 0.0; }
    self.last_elapsed = ctx.elapsed;
    let ax = ctx.width as f64 / (ctx.height as f64 * 2.0).max(1.0);
    while ctx.elapsed >= self.next_spawn {
      self.drops.push(Drop {
        x: self.rng.gen_range(0.08..0.92),
        y: self.rng.gen_range(0.08..0.92),
        t0: self.next_spawn,
        strength: self.rng.gen_range(0.7..1.0),
      });
      self.next_spawn += SPAWN_DT * self.rng.gen_range(0.6..1.5);
    }
    self.drops.retain(|d| ctx.elapsed - d.t0 < 7.0);
    while self.drops.len() > 16 { self.drops.remove(0); }
    let contrast = ctx.options.contrast;
    let mut grid = vec![0.0_f64; ctx.width * ctx.height];
    let dw = (ctx.width.saturating_sub(1)).max(1) as f64;
    let dh = (ctx.height.saturating_sub(1)).max(1) as f64;
    for row in 0..ctx.height {
      let v = row as f64 / dh;
      let base = row * ctx.width;
      for col in 0..ctx.width {
        let u = col as f64 / dw;
        let mut h = 0.0_f64;
        for d in &self.drops {
          let age = ctx.elapsed - d.t0;
          let dx = (u - d.x) * ax;
          let dy = v - d.y;
          let r = (dx * dx + dy * dy).sqrt();
          let front = WAVE_SPEED * age;
          let ring = r - front;
          let envelope = (-(ring * ring) / 0.010).exp();
          let decay = (-age * 0.5).exp() / (1.0 + 6.0 * r);
          h += d.strength * envelope * decay * (38.0 * ring).sin();
        }
        grid[base + col] = clamp(0.42 + h * 2.4 * contrast);
      }
    }
    render_field(ctx, &grid, TH, &STYLE, out);
  }
}
