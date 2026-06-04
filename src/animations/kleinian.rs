//! Limit set of a 2-generator quasi-Fuchsian Kleinian group, in the spirit
//! of Cannon-Thurston limit-set visualizations (Indra's Pearls).
//!
//! The actual Cannon-Thurston theorem describes a continuous extension of the
//! Cannon-Thurston map S^1 → S^2 for surface group inclusions into fibered
//! 3-manifold groups. The geometric content visible to a viewer is the
//! *image* of the boundary circle — a fractal curve on the sphere at
//! infinity. Implementing the actual CT setup requires picking a specific
//! fibered hyperbolic 3-manifold and tracking the surface-group action; we
//! cheat and just visualize the limit set of a 2-generator quasi-Fuchsian
//! Kleinian group, which is the same kind of fractal curve.
//!
//! Method (random orbit / "chaos game"):
//!   1. Two Möbius generators g1, g2 and their inverses form a 4-letter
//!      alphabet. The group they generate is free of rank 2.
//!   2. Apply a long random non-backtracking word to a seed point. Skip the
//!      first ~30 letters (transient) and splat every subsequent image.
//!   3. After many seeds, the union of splatted points densely traces the
//!      limit set.
//!
//! The group parameters drift slowly over time so the limit set morphs.
//! Persistent accumulator with exponential fade so each frame's freshly
//! drawn points add to a gradually-decaying impression.

use std::f64::consts::PI;

use rand::{Rng, SeedableRng};
use rand_pcg::Pcg32;

use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, render_field, FieldStyle};

const STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "infrared" };
const TH: &[(f64, char)] = &[
  (0.06, ' '),
  (0.14, '.'),
  (0.24, ':'),
  (0.36, '-'),
  (0.50, '='),
  (0.64, '+'),
  (0.78, '*'),
  (0.90, '#'),
  (1.01, '@'),
];

const N_SEEDS: usize = 80;
const WALK_LENGTH: usize = 120;
const TRANSIENT: usize = 30;
const FADE_RATE: f64 = 1.6;
const VIEW_SCALE: f64 = 0.35;

type Cx = (f64, f64);

#[inline]
fn cadd(a: Cx, b: Cx) -> Cx {
  (a.0 + b.0, a.1 + b.1)
}
#[inline]
fn cmul(a: Cx, b: Cx) -> Cx {
  (a.0 * b.0 - a.1 * b.1, a.0 * b.1 + a.1 * b.0)
}
#[inline]
fn cdiv(a: Cx, b: Cx) -> Cx {
  let denom = b.0 * b.0 + b.1 * b.1 + 1e-14;
  ((a.0 * b.0 + a.1 * b.1) / denom, (a.1 * b.0 - a.0 * b.1) / denom)
}

#[derive(Clone, Copy)]
struct Mob {
  a: Cx,
  b: Cx,
  c: Cx,
  d: Cx,
}

impl Mob {
  #[inline]
  fn apply(self, z: Cx) -> Cx {
    cdiv(cadd(cmul(self.a, z), self.b), cadd(cmul(self.c, z), self.d))
  }
  /// Inverse for SL(2, C) (or any Mob with ad-bc != 0; we scale up to a
  /// consistent normalization so the matrix entries do not blow up). For our
  /// drift-based generators ad-bc stays close to 1, so we just use the
  /// adjugate.
  #[inline]
  fn inverse(self) -> Mob {
    Mob { a: self.d, b: (-self.b.0, -self.b.1), c: (-self.c.0, -self.c.1), d: self.a }
  }
}

/// Build the two generators with time-varying parameters. The recipe is the
/// classic quasi-Fuchsian deformation of a 4-times-punctured sphere group:
/// two near-parabolic transformations whose conjugacy invariants we let
/// drift slightly to morph the limit set without losing its overall shape.
fn generators(t: f64) -> [Mob; 4] {
  let s = 1.0 + 0.18 * (t * 0.15).sin();
  let phi = 0.55 + 0.20 * (t * 0.11).cos();
  // g1: matrix [ s, 1; 0, 1/s ] — hyperbolic, fixed points 0 and ∞
  let g1 = Mob { a: (s, 0.0), b: (1.0, 0.0), c: (0.0, 0.0), d: (1.0 / s, 0.0) };
  // g2: conjugate g1 by a rotation+translation that moves fixed points
  // off the real axis. Result is a hyperbolic with two complex fixed
  // points -- the kind of conjugate that produces a quasi-Fuchsian limit
  // set.
  let cos_p = phi.cos();
  let sin_p = phi.sin();
  let rot = Mob { a: (cos_p, sin_p), b: (0.0, 0.0), c: (0.0, 0.0), d: (cos_p, -sin_p) };
  let trans = Mob { a: (1.0, 0.0), b: (0.4, 0.2), c: (0.0, 0.0), d: (1.0, 0.0) };
  // g2 = trans * rot * g1 * rot^{-1} * trans^{-1}
  let conj = compose(trans, rot);
  let conj_inv = compose(rot.inverse(), trans.inverse());
  let g2 = compose(compose(conj, g1), conj_inv);
  [g1, g1.inverse(), g2, g2.inverse()]
}

#[inline]
fn compose(a: Mob, b: Mob) -> Mob {
  // (a · b)(z) = a(b(z)); in matrix terms it's a * b.
  Mob {
    a: cadd(cmul(a.a, b.a), cmul(a.b, b.c)),
    b: cadd(cmul(a.a, b.b), cmul(a.b, b.d)),
    c: cadd(cmul(a.c, b.a), cmul(a.d, b.c)),
    d: cadd(cmul(a.c, b.b), cmul(a.d, b.d)),
  }
}

/// Which letter undoes letter `i`? Pairs are (0, 1) and (2, 3).
#[inline]
fn opposite(i: usize) -> usize {
  i ^ 1
}

pub struct Kleinian {
  accumulator: Vec<f64>,
  w: usize,
  h: usize,
  last_elapsed: f64,
  rng: Pcg32,
}

impl Default for Kleinian {
  fn default() -> Self {
    Self {
      accumulator: Vec::new(),
      w: 0,
      h: 0,
      last_elapsed: 0.0,
      rng: Pcg32::seed_from_u64(0xC1A6_D0CD_CAFE_BABE),
    }
  }
}

impl Animation for Kleinian {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    if ctx.width != self.w || ctx.height != self.h || ctx.elapsed < self.last_elapsed {
      self.accumulator = vec![0.0_f64; ctx.width * ctx.height];
      self.w = ctx.width;
      self.h = ctx.height;
    }
    let dt = (ctx.elapsed - self.last_elapsed).clamp(0.0, 0.2);
    self.last_elapsed = ctx.elapsed;
    let decay = (-dt * FADE_RATE).exp();
    for v in self.accumulator.iter_mut() {
      *v *= decay;
    }

    let w = ctx.width;
    let h = ctx.height;
    let ax = w as f64 / (h as f64 * 2.0).max(1.0);
    let dw = (w.saturating_sub(1)).max(1) as f64;
    let dh = (h.saturating_sub(1)).max(1) as f64;
    let t = ctx.elapsed;

    let gens = generators(t);

    for seed_idx in 0..N_SEEDS {
      // Spread the seed points around a circle in C so all four
      // generators get reached.
      let seed_angle = 2.0 * PI * seed_idx as f64 / N_SEEDS as f64;
      let mut z: Cx = (1.6 * seed_angle.cos(), 1.6 * seed_angle.sin());
      let mut prev: usize = usize::MAX;

      for step in 0..WALK_LENGTH {
        // Pick a non-backtracking letter.
        let mut next = self.rng.gen_range(0..4);
        if prev != usize::MAX && next == opposite(prev) {
          next = (next + 1) % 4;
          if prev != usize::MAX && next == opposite(prev) {
            next = (next + 1) % 4;
          }
        }
        z = gens[next].apply(z);
        prev = next;

        if step < TRANSIENT {
          continue;
        }

        // Disregard runaway points (orbit shot off to infinity).
        if !z.0.is_finite() || !z.1.is_finite() || z.0 * z.0 + z.1 * z.1 > 25.0 {
          break;
        }

        // Map complex plane to screen.
        let screen_u = 0.5 + VIEW_SCALE * z.0 / ax;
        let screen_v = 0.5 - VIEW_SCALE * z.1;
        if !(0.0..1.0).contains(&screen_u) || !(0.0..1.0).contains(&screen_v) {
          continue;
        }
        let cx_i = (screen_u * dw) as i64;
        let cy_i = (screen_v * dh) as i64;
        // 3x3 Gaussian splat -- single pixels look noisy, soft footprints
        // build into legible curves.
        for dy in -1..=1_i64 {
          let yy = cy_i + dy;
          if yy < 0 || yy >= h as i64 {
            continue;
          }
          for dx in -1..=1_i64 {
            let xx = cx_i + dx;
            if xx < 0 || xx >= w as i64 {
              continue;
            }
            let dist2 = (dx * dx + dy * dy) as f64;
            let g = (-dist2 / 0.65).exp();
            self.accumulator[yy as usize * w + xx as usize] += 0.10 * g;
          }
        }
      }
    }

    let contrast = ctx.options.contrast;
    let mut grid = vec![0.0_f64; w * h];
    for (g, &v) in grid.iter_mut().zip(self.accumulator.iter()) {
      *g = clamp(v * contrast);
    }
    render_field(ctx, &grid, TH, &STYLE, out);
  }
}
