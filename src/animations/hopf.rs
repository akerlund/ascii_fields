//! Hopf fibration — circle bundle S^1 → S^3 → S^2.
//!
//! For a uniform set of base points on S^2 (Fibonacci spiral so the
//! distribution does not collapse against the poles), we draw each base
//! point's Hopf fiber: the great circle in S^3 above it. Adjacent base
//! points have linked fibers (Hopf links), and as a family they nest into
//! tori filling space.
//!
//! Pipeline per fiber, per α-step:
//! 1. From a base point (X, Y, Z) ∈ S^2 compute the section angles
//!    cos²η = (1+Z)/2,  sin²η = (1-Z)/2,  ξ₁ - ξ₂ = atan2(Y, X)
//!    with ξ₁ + ξ₂ = 0 chosen for the section.
//! 2. Fiber point in S^3 (as ℝ^4):
//!    (cos η · cos(ξ₁+α), cos η · sin(ξ₁+α),
//!    sin η · cos(ξ₂+α), sin η · sin(ξ₂+α))
//! 3. Stereographic project S^3 → ℝ^3 from (0, 0, 0, 1). Points near the
//!    north pole shoot to infinity and are culled — those segments show
//!    up as gaps in the projected fiber, which is geometrically correct.
//! 4. Two-axis 3D rotation (viewpoint drift).
//! 5. Orthographic projection to screen + 3x3 Gaussian splat into a
//!    persistent accumulator that decays each frame, so the fibers trace
//!    fading worldlines.

use std::f64::consts::TAU;

use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, render_field, FieldStyle};

const STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "nebula" };
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

/// Per-second decay applied to the persistent accumulator.
const FADE_RATE: f64 = 1.5;
const N_FIBERS: usize = 64;
const PTS_PER_FIBER: usize = 100;
/// Drop fiber samples whose stereographic denominator is too small (close
/// to the projection's north-pole singularity).
const STEREO_CULL: f64 = 0.04;
/// Orthographic screen scale: roughly fits a Hopf fiber of unit radius in
/// the view window after stereographic projection.
const VIEW_SCALE: f64 = 0.28;

/// Uniform-ish point on S^2 from a Fibonacci spiral, with an extra azimuthal
/// rotation so the sampling drifts over time.
#[inline]
fn fib_sphere_point(i: usize, n: usize, rotation: f64) -> (f64, f64, f64) {
  let golden = std::f64::consts::PI * (3.0 - 5.0_f64.sqrt());
  let y = 1.0 - 2.0 * (i as f64 + 0.5) / n as f64;
  let r = (1.0 - y * y).sqrt();
  let theta = golden * i as f64 + rotation;
  (r * theta.cos(), y, r * theta.sin())
}

pub struct HopfFibration {
  accumulator: Vec<f64>,
  w: usize,
  h: usize,
  last_elapsed: f64,
}

impl Default for HopfFibration {
  fn default() -> Self {
    Self { accumulator: Vec::new(), w: 0, h: 0, last_elapsed: 0.0 }
  }
}

impl Animation for HopfFibration {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    // Reset on resize or time rewind.
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

    let base_rotation = t * 0.10;
    let rot_y = t * 0.22;
    let rot_x = t * 0.13;
    let (cy_rot, sy_rot) = (rot_y.cos(), rot_y.sin());
    let (cx_rot, sx_rot) = (rot_x.cos(), rot_x.sin());

    for fiber_idx in 0..N_FIBERS {
      let (s_x, s_y, s_z) = fib_sphere_point(fiber_idx, N_FIBERS, base_rotation);

      // Section angles. cos²η = (1+Z)/2 means cos_eta = sqrt((1+Z)/2).
      let cos_eta = ((1.0 + s_z) * 0.5).max(0.0).sqrt();
      let sin_eta = ((1.0 - s_z) * 0.5).max(0.0).sqrt();
      let xi_diff = s_y.atan2(s_x);
      let xi1 = xi_diff * 0.5;
      let xi2 = -xi_diff * 0.5;

      for k in 0..PTS_PER_FIBER {
        let alpha = TAU * k as f64 / PTS_PER_FIBER as f64;
        let beta1 = xi1 + alpha;
        let beta2 = xi2 + alpha;
        let x1 = cos_eta * beta1.cos();
        let x2 = cos_eta * beta1.sin();
        let x3 = sin_eta * beta2.cos();
        let x4 = sin_eta * beta2.sin();

        // Stereographic projection from (0, 0, 0, 1).
        let denom = 1.0 - x4;
        if denom.abs() < STEREO_CULL {
          continue;
        }
        let p1 = x1 / denom;
        let p2 = x2 / denom;
        let p3 = x3 / denom;

        // 3D rotation: around y then around x. Composed inline.
        let r1 = cy_rot * p1 + sy_rot * p3;
        let r3 = -sy_rot * p1 + cy_rot * p3;
        let r2 = p2;
        let s2 = cx_rot * r2 - sx_rot * r3;
        let s3 = sx_rot * r2 + cx_rot * r3;
        let s1 = r1;

        // Orthographic project to screen.
        let screen_u = 0.5 + VIEW_SCALE * s1 / ax;
        let screen_v = 0.5 - VIEW_SCALE * s2;
        if !(0.0..1.0).contains(&screen_u) || !(0.0..1.0).contains(&screen_v) {
          continue;
        }
        // Depth fade so far points dim, near ones bright.
        let depth_fade = 1.0 / (1.0 + 0.35 * s3 * s3);
        let weight = 0.16 * depth_fade;

        let cx_i = (screen_u * dw) as i64;
        let cy_i = (screen_v * dh) as i64;
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
            self.accumulator[yy as usize * w + xx as usize] += weight * g;
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
