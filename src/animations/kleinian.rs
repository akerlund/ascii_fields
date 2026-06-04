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

const N_SEEDS: usize = 120;
const WALK_LENGTH: usize = 180;
const TRANSIENT: usize = 40;
const FADE_RATE: f64 = 1.4;
/// The Maskit-slice limit set lives roughly in the strip |Re z| < 2,
/// |Im z| < 1.5. Pick a screen scale that fits that without clipping.
const VIEW_SCALE: f64 = 0.20;

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

/// Build the two generators using the Maskit-slice parameterization for the
/// once-punctured torus group. For a complex parameter μ:
///
///   T_a = ((−iμ − 1, −iμ), (i, 0))
///   T_b = ((1, −2i), (0, 1))
///
/// T_b is parabolic at infinity (translation by −2i). T_a is parabolic
/// along the Maskit boundary; μ drifts slowly inside the slice so the
/// limit set deforms continuously without leaving the discrete-group
/// regime.
fn generators(t: f64) -> [Mob; 4] {
  // Maskit slice parameter -- staying around μ ≈ 1.9 + 0.05i keeps us inside
  // the discrete locus where the limit set is a beautiful Apollonian-ish
  // fractal curve rather than the degenerate two-attractor regime.
  let mu_re = 1.92 + 0.06 * (t * 0.08).sin();
  let mu_im = 0.05 + 0.04 * (t * 0.11).cos();
  // i · μ = (−mu_im, mu_re)
  let i_mu = (-mu_im, mu_re);
  let neg_i_mu = (mu_im, -mu_re);
  let neg_i_mu_minus_1 = (neg_i_mu.0 - 1.0, neg_i_mu.1);

  let g1 = Mob { a: neg_i_mu_minus_1, b: neg_i_mu, c: (0.0, 1.0), d: (0.0, 0.0) };
  // T_b = ((1, -2i), (0, 1)).
  let g2 = Mob { a: (1.0, 0.0), b: (0.0, -2.0), c: (0.0, 0.0), d: (1.0, 0.0) };

  // Silence the dead-code warning on the now-unused `i_mu` value -- kept
  // visible above so the slice formula reads cleanly.
  let _ = i_mu;

  [g1, g1.inverse(), g2, g2.inverse()]
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
      // Spread seed points across the strip where the Maskit-slice limit
      // set lives. A wide rectangle (re ∈ [-2, 2], im ∈ [-1.5, 1.5])
      // makes sure every iterate region of the limit set has a starter.
      let su = (seed_idx as f64 + 0.5) / N_SEEDS as f64;
      let re = -2.0 + 4.0 * (su * 7.0).fract();
      let im = -1.4 + 2.8 * su;
      let mut z: Cx = (re, im);
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
