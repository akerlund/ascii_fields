use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, render_field, FieldStyle};
use crate::noise::fbm;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg32;
use std::f64::consts::TAU;

const STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "nebula" };
const TH: &[(f64, char)] = &[
  (0.10, ' '),
  (0.20, '.'),
  (0.32, ':'),
  (0.44, '-'),
  (0.56, '='),
  (0.68, '+'),
  (0.80, '*'),
  (0.90, '#'),
  (1.01, '@'),
];
const SPAWN_DT: f64 = 0.22;

#[derive(Clone, Copy)]
struct Pair {
  x: f64,
  y: f64,
  angle: f64,
  t0: f64,
  life: f64,
  reach: f64,
}

pub struct QField {
  pairs: Vec<Pair>,
  next: f64,
  last: f64,
  rng: Pcg32,
}
impl Default for QField {
  fn default() -> Self {
    Self { pairs: Vec::new(), next: 0.0, last: 0.0, rng: Pcg32::from_entropy() }
  }
}

impl Animation for QField {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    if ctx.elapsed < self.last {
      self.pairs.clear();
      self.next = 0.0;
    }
    self.last = ctx.elapsed;
    while ctx.elapsed >= self.next {
      self.pairs.push(Pair {
        x: self.rng.gen_range(0.05..0.95),
        y: self.rng.gen_range(0.05..0.95),
        angle: self.rng.gen_range(0.0..TAU),
        t0: self.next,
        life: self.rng.gen_range(0.5..1.1),
        reach: self.rng.gen_range(0.04..0.10),
      });
      self.next += SPAWN_DT * self.rng.gen_range(0.4..1.6);
    }
    self.pairs.retain(|p| ctx.elapsed - p.t0 < p.life);
    let ax = ctx.width as f64 / (ctx.height as f64 * 2.0).max(1.0);
    let contrast = ctx.options.contrast;
    let mut active: Vec<(f64, f64, f64, f64, f64)> = Vec::with_capacity(self.pairs.len());
    for p in &self.pairs {
      let age = (ctx.elapsed - p.t0) / p.life;
      let sep = (age * std::f64::consts::PI).sin() * p.reach;
      let (ca, sa) = (p.angle.cos(), p.angle.sin());
      let mut intensity = 0.5 + 0.5 * (age * std::f64::consts::PI).sin();
      if age > 0.85 {
        intensity += (age - 0.85) / 0.15;
      }
      active.push((p.x + ca * sep, p.y + sa * sep, p.x - ca * sep, p.y - sa * sep, intensity));
    }
    let mut grid = vec![0.0_f64; ctx.width * ctx.height];
    let dw = (ctx.width.saturating_sub(1)).max(1) as f64;
    let dh = (ctx.height.saturating_sub(1)).max(1) as f64;
    for row in 0..ctx.height {
      let v = row as f64 / dh;
      let base = row * ctx.width;
      for col in 0..ctx.width {
        let u = col as f64 / dw;
        let field = 0.14 + 0.30 * fbm(u * 6.0 + ctx.elapsed * 0.4, v * 6.0 - ctx.elapsed * 0.3, 4);
        let mut spark = 0.0_f64;
        for &(ax_, ay_, bx_, by_, inten) in &active {
          let dxa = (u - ax_) * ax;
          let dya = v - ay_;
          let dxb = (u - bx_) * ax;
          let dyb = v - by_;
          spark += inten * (-(dxa * dxa + dya * dya) / 0.0008).exp();
          spark += inten * (-(dxb * dxb + dyb * dyb) / 0.0008).exp();
        }
        grid[base + col] = clamp((field + spark) * contrast);
      }
    }
    render_field(ctx, &grid, TH, &STYLE, out);
  }
}
