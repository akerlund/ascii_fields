//! Animated Sierpinski triangle drawn via the chaos game over three rotating
//! vertices.
//!
//! Previous behaviour: a static carpet membership-test rendered as a slow
//! zoom into a corner. Pretty but motionless -- user described it as boring.
//!
//! New approach: three vertices arranged on a circle that slowly rotates as
//! a rigid body. The chaos-game iteration `p_{n+1} = (p_n + v_i) / 2` with
//! `v_i` picked uniformly at random produces a sequence of points dense in
//! the Sierpinski triangle. We splat ~10k points per frame into a persistent
//! accumulator that exponentially decays each frame, so the triangle traces
//! out as a continuous sweep of fading points; rotating the vertices then
//! turns the whole triangle into a slowly spinning attractor.

use std::f64::consts::TAU;

use rand::{Rng, SeedableRng};
use rand_pcg::Pcg32;

use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, render_field, FieldStyle};

const STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "amber" };
const TH: &[(f64, char)] = &[
  (0.06, ' '),
  (0.16, '.'),
  (0.26, ':'),
  (0.38, '-'),
  (0.50, '='),
  (0.62, '+'),
  (0.76, '*'),
  (0.88, '#'),
  (1.01, '@'),
];

const ITERATIONS_PER_FRAME: usize = 12_000;
const FADE_RATE: f64 = 1.8;

pub struct Sierpinski {
  accumulator: Vec<f64>,
  w: usize,
  h: usize,
  last_elapsed: f64,
  point: (f64, f64),
  rng: Pcg32,
}

impl Default for Sierpinski {
  fn default() -> Self {
    Self {
      accumulator: Vec::new(),
      w: 0,
      h: 0,
      last_elapsed: 0.0,
      point: (0.5, 0.5),
      rng: Pcg32::seed_from_u64(0x51E5_71F1_A570),
    }
  }
}

impl Animation for Sierpinski {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let w = ctx.width;
    let h = ctx.height;
    if ctx.elapsed < self.last_elapsed {
      // Rewind: full reset.
      self.accumulator = vec![0.0_f64; w * h];
      self.w = w;
      self.h = h;
      self.point = (0.5, 0.5);
    } else if w != self.w || h != self.h {
      // Resize: rebuild the accumulator buffer but preserve the chaos-
      // game iterate so the point of the next splat continues from where
      // it was.
      self.accumulator = vec![0.0_f64; w * h];
      self.w = w;
      self.h = h;
    }
    let dt = (ctx.elapsed - self.last_elapsed).clamp(0.0, 0.2);
    self.last_elapsed = ctx.elapsed;
    let decay = (-dt * FADE_RATE).exp();
    for v in self.accumulator.iter_mut() {
      *v *= decay;
    }

    let ax = w as f64 / (h as f64 * 2.0).max(1.0);
    let dw = (w.saturating_sub(1)).max(1) as f64;
    let dh = (h.saturating_sub(1)).max(1) as f64;
    let t = ctx.elapsed;

    // Three rotating vertices. The triangle's circumradius pulses slightly
    // so the whole figure breathes as it spins.
    let theta = t * 0.18;
    let r_circle = 0.42 + 0.03 * (t * 0.10).sin();
    let v0 = vertex(theta, r_circle, ax);
    let v1 = vertex(theta + TAU / 3.0, r_circle, ax);
    let v2 = vertex(theta + 2.0 * TAU / 3.0, r_circle, ax);
    let vertices = [v0, v1, v2];

    // Chaos game.
    let mut p = self.point;
    for _ in 0..ITERATIONS_PER_FRAME {
      let i = self.rng.gen_range(0..3);
      let v = vertices[i];
      p.0 = 0.5 * (p.0 + v.0);
      p.1 = 0.5 * (p.1 + v.1);

      let col = (p.0 * dw) as i64;
      let row = (p.1 * dh) as i64;
      // 3x3 Gaussian splat: single pixels often fall below the ' '
      // threshold and the figure looks sparse.
      for dy in -1..=1_i64 {
        let yy = row + dy;
        if yy < 0 || yy >= h as i64 {
          continue;
        }
        for dx in -1..=1_i64 {
          let xx = col + dx;
          if xx < 0 || xx >= w as i64 {
            continue;
          }
          let dist2 = (dx * dx + dy * dy) as f64;
          let g = (-dist2 / 0.65).exp();
          self.accumulator[yy as usize * w + xx as usize] += 0.022 * g;
        }
      }
    }
    self.point = p;

    // Log tone-map so the centre does not saturate while dim outer reaches
    // still register.
    let contrast = ctx.options.contrast;
    let k: f64 = 6.0;
    let norm = (1.0 + k).ln();
    let mut grid = vec![0.0_f64; w * h];
    for (g, &v) in grid.iter_mut().zip(self.accumulator.iter()) {
      let compressed = (1.0 + k * v).ln() / norm;
      *g = clamp(compressed * contrast);
    }
    render_field(ctx, &grid, TH, &STYLE, out);
  }
}

/// Vertex at angle `theta` on a circle of radius `r_circle` centred at
/// (0.5, 0.5) in screen coordinates. `ax` is the standard 2:1 cell-aspect
/// correction so the triangle looks geometrically regular instead of
/// stretched horizontally.
#[inline]
fn vertex(theta: f64, r_circle: f64, ax: f64) -> (f64, f64) {
  (0.5 + (r_circle * theta.cos()) / ax, 0.5 + r_circle * theta.sin())
}
