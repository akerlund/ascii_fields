//! Roots of monic cubics with the linear coefficient restricted to the eight
//! 8th roots of unity. We sweep many cubics per frame, solve each numerically,
//! and splat the three (possibly complex) roots into a persistent accumulator
//! that exponentially decays each frame -- so the roots trace fading curves
//! through the complex plane as the family parameters drift.
//!
//! Cubic to solve: f(x) = x^3 + b*x^2 + c*x + d
//! with c restricted to {exp(2πik/8) : k = 0..7}.
//! b and d both sweep slowly in time so the root locus evolves continuously.
//!
//! Solver is Durand-Kerner: three guesses converge to all three roots in ~6-10
//! iterations from a triangular initial layout that avoids the polynomial's
//! own symmetries.

use std::f64::consts::TAU;

use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, render_field, FieldStyle};

const STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "spectrum" };
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

/// Visible window in the complex plane: roots within +/-VIEW_RADIUS in either
/// axis are plotted; outside roots are silently skipped.
const VIEW_RADIUS: f64 = 1.9;

const C_ROOTS: usize = 8;
const SAMPLES_PER_C: usize = 36;
const DK_ITERATIONS: usize = 9;

/// Exponential decay constant applied to the accumulator each second. Higher
/// = faster fade, less trailing.
const FADE_RATE: f64 = 2.2;

type Cx = (f64, f64);

#[inline]
fn cadd(a: Cx, b: Cx) -> Cx {
  (a.0 + b.0, a.1 + b.1)
}
#[inline]
fn csub(a: Cx, b: Cx) -> Cx {
  (a.0 - b.0, a.1 - b.1)
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

/// Evaluate p(x) = x^3 + b*x^2 + c*x + d at x.
#[inline]
fn poly(x: Cx, b: Cx, c: Cx, d: Cx) -> Cx {
  let x2 = cmul(x, x);
  let x3 = cmul(x2, x);
  cadd(cadd(cadd(x3, cmul(b, x2)), cmul(c, x)), d)
}

fn solve_cubic(b: Cx, c: Cx, d: Cx) -> [Cx; 3] {
  // Initial guesses on a non-degenerate triangle. Anchored away from any
  // particular root of unity so generic inputs converge cleanly.
  let mut r: [Cx; 3] = [(0.41, 0.93), (-0.87, -0.51), (0.49, -0.37)];
  for _ in 0..DK_ITERATIONS {
    let p0 = poly(r[0], b, c, d);
    let p1 = poly(r[1], b, c, d);
    let p2 = poly(r[2], b, c, d);
    let den0 = cmul(csub(r[0], r[1]), csub(r[0], r[2]));
    let den1 = cmul(csub(r[1], r[0]), csub(r[1], r[2]));
    let den2 = cmul(csub(r[2], r[0]), csub(r[2], r[1]));
    r[0] = csub(r[0], cdiv(p0, den0));
    r[1] = csub(r[1], cdiv(p1, den1));
    r[2] = csub(r[2], cdiv(p2, den2));
  }
  r
}

pub struct CubicRoots {
  accumulator: Vec<f64>,
  w: usize,
  h: usize,
  last_elapsed: f64,
}

impl Default for CubicRoots {
  fn default() -> Self {
    Self { accumulator: Vec::new(), w: 0, h: 0, last_elapsed: 0.0 }
  }
}

impl Animation for CubicRoots {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    // Reseed buffer on resize or time rewind.
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

    // For each of the eight roots of unity, sweep a family of (b, d) values
    // and splat the three roots of each resulting cubic.
    for c_idx in 0..C_ROOTS {
      let c_phase = TAU * c_idx as f64 / C_ROOTS as f64 + t * 0.05;
      let c = (c_phase.cos(), c_phase.sin());

      for k in 0..SAMPLES_PER_C {
        let frac = k as f64 / SAMPLES_PER_C as f64;
        let b_phi = TAU * frac + t * 0.21 + c_idx as f64 * 0.32;
        let b_r = 0.85 + 0.45 * (t * 0.11).sin();
        let b = (b_r * b_phi.cos(), b_r * b_phi.sin());
        let d_phi = TAU * frac * 1.7 + t * 0.13 + c_idx as f64 * 0.71;
        let d_r = 0.65 + 0.35 * (t * 0.07 + c_idx as f64).cos();
        let d = (d_r * d_phi.cos(), d_r * d_phi.sin());

        let roots = solve_cubic(b, c, d);
        for &root in &roots {
          // Map complex plane -> screen with horizontal aspect correction.
          let u = 0.5 + root.0 / (2.0 * VIEW_RADIUS * ax);
          let v = 0.5 - root.1 / (2.0 * VIEW_RADIUS);
          if !(0.0..1.0).contains(&u) || !(0.0..1.0).contains(&v) {
            continue;
          }
          let cx = (u * dw) as i64;
          let cy = (v * dh) as i64;
          // 3x3 Gaussian splat so single roots show up as a small soft dot.
          for dy in -1..=1_i64 {
            let yy = cy + dy;
            if yy < 0 || yy >= h as i64 {
              continue;
            }
            for dx in -1..=1_i64 {
              let xx = cx + dx;
              if xx < 0 || xx >= w as i64 {
                continue;
              }
              let dist2 = (dx * dx + dy * dy) as f64;
              let weight = (-dist2 / 0.65).exp();
              self.accumulator[yy as usize * w + xx as usize] += weight * 0.18;
            }
          }
        }
      }
    }

    // Final tone-map plus a very faint inside-disk vignette so completely
    // dark regions still hint at the unit disk.
    let contrast = ctx.options.contrast;
    let mut grid = vec![0.0_f64; w * h];
    for row in 0..h {
      let v = row as f64 / dh - 0.5;
      let base = row * w;
      for col in 0..w {
        let u = (col as f64 / dw - 0.5) * ax;
        let r = (u * u + v * v).sqrt();
        let vignette = 0.025 * (1.0 - r * 1.2).max(0.0);
        grid[base + col] = clamp((self.accumulator[base + col] + vignette) * contrast);
      }
    }
    render_field(ctx, &grid, TH, &STYLE, out);
  }
}
