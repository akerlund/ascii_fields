//! Stylized visualization of a sphere turning inside-out.
//!
//! The "real" mathematical sphere eversion (Smale 1958) deforms a sphere to
//! its mirror image through immersions, with no creases or tears. The
//! cleanest explicit construction is the Morin / Bryant half-way model
//! (Boy's surface), but writing out a regular homotopy through it takes
//! hundreds of lines of careful parameterization.
//!
//! This module renders a *stylized* eversion: a parameterized surface that
//! starts as a sphere at t=0, bulges into a self-intersecting Boy's-surface-
//! like shape at t=0.5, and ends as an inside-out sphere at t=1, then
//! reverses. The deformation is not a regular homotopy (it pinches), so
//! mathematicians, please look away. Visually the effect of a sphere
//! pulsing through its own surface comes across.
//!
//! Rendering follows the same pipeline as `hopf`: parameterize the surface,
//! apply 3D rotation, orthographic project, splat into a persistent fading
//! accumulator so each frame's surface leaves a brief trail.

use std::f64::consts::{PI, TAU};

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

/// Number of latitude circles rendered (u-coordinate stops). Sparse: with
/// dense sampling the surface paints as a featureless silhouette; with
/// sparse wireframe the deformation of each circle is legible.
const N_LATITUDES: usize = 14;
/// Number of meridians (v-coordinate stops).
const N_MERIDIANS: usize = 20;
/// Points sampled along each grid line so the curves draw smoothly.
const PTS_PER_LINE: usize = 200;
const FADE_RATE: f64 = 2.4;
const VIEW_SCALE: f64 = 0.30;

/// Point on the stylized eversion surface at parameters (u, v) and homotopy
/// time t in [0, 1]. t = 0 is a standard sphere, t = 1 is the radially
/// inverted sphere, and t = 0.5 is a self-intersecting half-way shape with
/// three-fold symmetric distortion (visually evoking Boy's surface).
fn eversion_point(u: f64, v: f64, t: f64) -> (f64, f64, f64) {
  let su = u.sin();
  let cu = u.cos();
  let sv = v.sin();
  let cv = v.cos();

  // Bulge envelope: zero at the endpoints, maximal at t=0.5.
  let bulge = (PI * t).sin();
  // Z-direction flip: cos(πt) goes from +1 to -1 as t: 0 → 1, exactly the
  // axial inversion that turns the sphere inside-out.
  let z_flip = (PI * t).cos();

  // Equatorial radius pumped by a three-fold-symmetric ripple at half-way.
  // The (3v) frequency is what makes the half-way surface look like the
  // three-petalled Boy's surface even though our parameterization is
  // not the actual Bryant section.
  let fold_radius = 1.0 + 0.6 * bulge * (3.0 * v + PI * t).sin();
  let r = su * fold_radius;
  let x = r * cv;
  let y = r * sv;
  // Add a polar bulge so the poles also push through the equator at
  // half-way; otherwise the surface only deforms at the waist.
  let polar_bulge = 0.4 * bulge * (2.0 * v).cos() * su.powi(2);
  let z = cu * z_flip + polar_bulge;
  (x, y, z)
}

pub struct SphereEversion {
  accumulator: Vec<f64>,
  w: usize,
  h: usize,
  last_elapsed: f64,
}

impl Default for SphereEversion {
  fn default() -> Self {
    Self { accumulator: Vec::new(), w: 0, h: 0, last_elapsed: 0.0 }
  }
}

impl Animation for SphereEversion {
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

    // Triangle wave 0 → 1 → 0 → 1 ..., one full cycle every ~11 seconds.
    let phase = (ctx.elapsed * 0.18) % 2.0;
    let t = if phase < 1.0 { phase } else { 2.0 - phase };

    let rot_y = ctx.elapsed * 0.20;
    let rot_x = 0.4 + 0.2 * (ctx.elapsed * 0.13).sin();
    let (cy_rot, sy_rot) = (rot_y.cos(), rot_y.sin());
    let (cx_rot, sx_rot) = (rot_x.cos(), rot_x.sin());

    // Reusable closure: project (p1, p2, p3) to screen and splat.
    let mut splat = |p1: f64, p2: f64, p3: f64| {
      let r1 = cy_rot * p1 + sy_rot * p3;
      let r3 = -sy_rot * p1 + cy_rot * p3;
      let r2 = p2;
      let s1 = r1;
      let s2 = cx_rot * r2 - sx_rot * r3;
      let s3 = sx_rot * r2 + cx_rot * r3;
      let screen_u = 0.5 + VIEW_SCALE * s1 / ax;
      let screen_v = 0.5 - VIEW_SCALE * s2;
      if !(0.0..1.0).contains(&screen_u) || !(0.0..1.0).contains(&screen_v) {
        return;
      }
      let depth_fade = 1.0 / (1.0 + 0.5 * s3 * s3);
      let weight = 0.30 * depth_fade;
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
          let g = (-dist2 / 0.55).exp();
          self.accumulator[yy as usize * w + xx as usize] += weight * g;
        }
      }
    };

    // Latitude lines: fix u, sweep v densely.
    for i in 0..N_LATITUDES {
      let u = PI * (i as f64 + 0.5) / N_LATITUDES as f64;
      for j in 0..PTS_PER_LINE {
        let v = TAU * j as f64 / PTS_PER_LINE as f64;
        let (p1, p2, p3) = eversion_point(u, v, t);
        splat(p1, p2, p3);
      }
    }
    // Meridian lines: fix v, sweep u densely. Together they form a
    // wireframe whose grid lines bend through the eversion.
    for j in 0..N_MERIDIANS {
      let v = TAU * j as f64 / N_MERIDIANS as f64;
      for i in 0..PTS_PER_LINE {
        let u = PI * (i as f64 + 0.5) / PTS_PER_LINE as f64;
        let (p1, p2, p3) = eversion_point(u, v, t);
        splat(p1, p2, p3);
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
