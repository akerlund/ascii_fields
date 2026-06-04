//! Möbius transformations on the Poincaré disk — three one-parameter families
//! with distinct geometric character. All three apply the same idea: inverse
//! Möbius map per screen pixel, sample a structured base pattern at the
//! preimage. The choice of family determines what the flow looks like.
//!
//! Conventions:
//!   * Coordinates are scaled so the unit disk fits the screen with the same
//!     aspect ratio used everywhere else (terminal cells are roughly 2:1).
//!   * All three end with a soft mask just inside |z| = 1 so the disk
//!     boundary reads as a ring instead of a hard cut.
//!
//! The three variants:
//!
//! **Mobius0 — elliptic**
//!   T(z) = exp(i·θ) · (z - a) / (1 - ā·z), pole `a` orbits inside the disk
//!   and θ drifts linearly. Hyperbolic "panning" toward / away from a moving
//!   point with an overall swirl.
//!
//! **Mobius1 — hyperbolic translation**
//!   Pure sliding along a (slowly rotating) geodesic. Two boundary fixed
//!   points; everything in the interior flows from one to the other. The
//!   visual: streamlines that form circular arcs between two anti-podal
//!   points on the unit circle.
//!
//! **Mobius2 — loxodromic spiral**
//!   Hyperbolic translation composed with a rotation that does not share
//!   the same axis, so the streamlines twist into logarithmic spirals
//!   wrapped around the disk's two attractor / repeller points.

use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, render_field, FieldStyle};
use crate::noise::fbm;

const STYLE_ELLIPTIC: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "plasma" };
const STYLE_HYPERBOLIC: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "aurora" };
const STYLE_LOXODROMIC: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "stellar" };

const TH: &[(f64, char)] = &[
  (0.10, ' '),
  (0.22, '.'),
  (0.34, ':'),
  (0.46, '-'),
  (0.58, '='),
  (0.70, '+'),
  (0.82, '*'),
  (0.92, '#'),
  (1.01, '@'),
];

/// Smooth fade just inside the unit disk boundary so |z| = 1 reads as a ring.
#[inline]
fn disk_mask(pr: f64) -> f64 {
  (1.0 - (pr - 0.92) / 0.10).clamp(0.0, 1.0)
}

/// Apply the inverse of the elliptic Möbius automorphism
/// T(z) = exp(i·θ) · (z - a) / (1 - ā·z), returning the preimage of (px, py)
/// for a pole a = (ax_pole, ay_pole) and rotation θ.
#[inline]
fn inv_elliptic(px: f64, py: f64, ax_pole: f64, ay_pole: f64, theta: f64) -> (f64, f64) {
  let num_x = px - ax_pole;
  let num_y = py - ay_pole;
  let den_x = 1.0 - (ax_pole * px + ay_pole * py);
  let den_y = ay_pole * px - ax_pole * py;
  let denom = den_x * den_x + den_y * den_y + 1e-9;
  let zx = (num_x * den_x + num_y * den_y) / denom;
  let zy = (num_y * den_x - num_x * den_y) / denom;
  let (ctheta, stheta) = (theta.cos(), theta.sin());
  (zx * ctheta - zy * stheta, zx * stheta + zy * ctheta)
}

/// Hyperbolic translation by parameter `a` (real, |a| < 1) along the x-axis
/// of the disk. Returns the preimage of (px, py).
#[inline]
fn inv_hyperbolic_x(px: f64, py: f64, a: f64) -> (f64, f64) {
  let num_x = px - a;
  let num_y = py;
  let den_x = 1.0 - a * px;
  let den_y = -a * py;
  let denom = den_x * den_x + den_y * den_y + 1e-9;
  ((num_x * den_x + num_y * den_y) / denom, (num_y * den_x - num_x * den_y) / denom)
}

#[inline]
fn rotate(px: f64, py: f64, angle: f64) -> (f64, f64) {
  let c = angle.cos();
  let s = angle.sin();
  (c * px - s * py, s * px + c * py)
}

// ---------------------------------------------------------------------------
// Mobius0: elliptic disk automorphism with moving pole.
// ---------------------------------------------------------------------------

pub struct Mobius0;

impl Animation for Mobius0 {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let w = ctx.width;
    let h = ctx.height;
    let ax = w as f64 / (h as f64 * 2.0).max(1.0);
    let contrast = ctx.options.contrast;
    let t = ctx.elapsed;

    let a_radius = (0.55 + 0.15 * (t * 0.11).sin()) * 0.55;
    let a_phase = t * 0.18;
    let ax_pole = a_radius * a_phase.cos();
    let ay_pole = a_radius * a_phase.sin();
    let theta = t * 0.42;

    let mut grid = vec![0.0_f64; w * h];
    let dw = (w.saturating_sub(1)).max(1) as f64;
    let dh = (h.saturating_sub(1)).max(1) as f64;

    for row in 0..h {
      let py = (row as f64 / dh - 0.5) * 2.0;
      let base = row * w;
      for col in 0..w {
        let px = (col as f64 / dw - 0.5) * 2.0 * ax;
        let (zx, zy) = inv_elliptic(px, py, ax_pole, ay_pole, theta);

        let r = (zx * zx + zy * zy).sqrt();
        let phi = zy.atan2(zx);
        let rings = 0.5 + 0.5 * (r * 13.0).sin();
        let spokes = 0.5 + 0.5 * (phi * 6.0).cos();
        let texture = fbm(zx * 3.0, zy * 3.0, 4);
        let pattern = rings * 0.45 + spokes * 0.25 + texture * 0.40;

        let pr = (px * px + py * py).sqrt();
        grid[base + col] = clamp(pattern * disk_mask(pr) * 1.45 * contrast);
      }
    }
    render_field(ctx, &grid, TH, &STYLE_ELLIPTIC, out);
  }
}

// ---------------------------------------------------------------------------
// Mobius1: hyperbolic translation along a slowly rotating geodesic.
// ---------------------------------------------------------------------------

pub struct Mobius1;

impl Animation for Mobius1 {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let w = ctx.width;
    let h = ctx.height;
    let ax = w as f64 / (h as f64 * 2.0).max(1.0);
    let contrast = ctx.options.contrast;
    let t = ctx.elapsed;

    // Axis angle: which boundary points (re^iφ at boundary) are the
    // attractor/repeller. Slow rotation so the flow direction drifts.
    let axis_angle = t * 0.10;
    // Translation parameter `a` oscillates in (-0.7, 0.7) so the flow
    // reverses direction periodically -- nicer than a monotone slide.
    let a = 0.65 * (t * 0.20).sin();

    let mut grid = vec![0.0_f64; w * h];
    let dw = (w.saturating_sub(1)).max(1) as f64;
    let dh = (h.saturating_sub(1)).max(1) as f64;

    for row in 0..h {
      let py = (row as f64 / dh - 0.5) * 2.0;
      let base = row * w;
      for col in 0..w {
        let px = (col as f64 / dw - 0.5) * 2.0 * ax;
        // Rotate so geodesic axis is the real line.
        let (rx, ry) = rotate(px, py, -axis_angle);
        // Hyperbolic untranslate by -a.
        let (zx, zy) = inv_hyperbolic_x(rx, ry, a);
        // Rotate back so the pattern is sampled in the original frame.
        let (sx, sy) = rotate(zx, zy, axis_angle);

        // Pattern: streaks aligned roughly with the flow direction make
        // the translation visible. Use a periodic "stripe" function along
        // the rotated coordinate combined with fbm for texture.
        let along = rx; // already in rotated frame
        let across = ry;
        let streaks = 0.5 + 0.5 * (along * 9.0 + t * 0.4).sin();
        let bands = 0.5 + 0.5 * (across * 7.0).cos();
        let texture = fbm(sx * 2.5, sy * 2.5, 4);
        let pattern = streaks * 0.45 + bands * 0.20 + texture * 0.45;

        let pr = (px * px + py * py).sqrt();
        grid[base + col] = clamp(pattern * disk_mask(pr) * 1.45 * contrast);
      }
    }
    render_field(ctx, &grid, TH, &STYLE_HYPERBOLIC, out);
  }
}

// ---------------------------------------------------------------------------
// Mobius2: loxodromic — hyperbolic translation + non-aligned rotation gives
// logarithmic spirals between the two boundary fixed points.
// ---------------------------------------------------------------------------

pub struct Mobius2;

impl Animation for Mobius2 {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let w = ctx.width;
    let h = ctx.height;
    let ax = w as f64 / (h as f64 * 2.0).max(1.0);
    let contrast = ctx.options.contrast;
    let t = ctx.elapsed;

    let axis_angle = t * 0.07;
    let a = 0.55 * (t * 0.18).sin();
    // The extra rotation is what makes the flow loxodromic instead of
    // purely hyperbolic. Different rate from axis_angle so it does not
    // collapse back to elliptic.
    let twist = t * 0.55;

    let mut grid = vec![0.0_f64; w * h];
    let dw = (w.saturating_sub(1)).max(1) as f64;
    let dh = (h.saturating_sub(1)).max(1) as f64;

    for row in 0..h {
      let py = (row as f64 / dh - 0.5) * 2.0;
      let base = row * w;
      for col in 0..w {
        let px = (col as f64 / dw - 0.5) * 2.0 * ax;
        // Inverse loxodromic: undo rotation by `twist`, undo hyperbolic
        // translation by `a` along the rotated geodesic.
        let (rx, ry) = rotate(px, py, -axis_angle);
        let (rrx, rry) = rotate(rx, ry, -twist);
        let (zx, zy) = inv_hyperbolic_x(rrx, rry, a);

        // Pattern: log-polar grid in the preimage emphasises the spiral
        // structure of the orbit.
        let r = (zx * zx + zy * zy).sqrt().max(1e-4);
        let phi = zy.atan2(zx);
        let log_r = r.ln();
        let spirals = 0.5 + 0.5 * (log_r * 6.0 + phi * 5.0).sin();
        let rings = 0.5 + 0.5 * (log_r * 11.0 - t * 0.2).cos();
        let texture = fbm(zx * 2.5, zy * 2.5, 3);
        let pattern = spirals * 0.45 + rings * 0.20 + texture * 0.45;

        let pr = (px * px + py * py).sqrt();
        grid[base + col] = clamp(pattern * disk_mask(pr) * 1.45 * contrast);
      }
    }
    render_field(ctx, &grid, TH, &STYLE_LOXODROMIC, out);
  }
}

// ---------------------------------------------------------------------------
// Mobius3: bend a flat 2D checkerboard plane onto a 3D sphere via inverse
// stereographic projection, then apply a Möbius transformation of the plane
// (equivalently, a 3D rotation of the sphere) so the bent grid rolls.
// ---------------------------------------------------------------------------

const STYLE_PLANE_BEND: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "ocean" };

/// Inverse stereographic projection ℝ² → S² ⊂ ℝ³ from the south pole:
/// (X, Y) → (2X, 2Y, X²+Y² − 1) / (X²+Y² + 1)
#[inline]
fn inverse_stereographic(x: f64, y: f64) -> (f64, f64, f64) {
  let denom = x * x + y * y + 1.0;
  (2.0 * x / denom, 2.0 * y / denom, (x * x + y * y - 1.0) / denom)
}

/// Forward stereographic projection S² → ℝ² from the north pole.
/// (X, Y, Z) → (X, Y) / (1 − Z)
#[inline]
fn stereographic(x: f64, y: f64, z: f64) -> Option<(f64, f64)> {
  let denom = 1.0 - z;
  if denom.abs() < 1e-4 {
    return None;
  }
  Some((x / denom, y / denom))
}

pub struct Mobius3;

impl Animation for Mobius3 {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let w = ctx.width;
    let h = ctx.height;
    let ax = w as f64 / (h as f64 * 2.0).max(1.0);
    let contrast = ctx.options.contrast;
    let t = ctx.elapsed;

    // 3D rotation of the sphere: equivalent to a Möbius transformation
    // of the underlying plane via the Riemann-sphere correspondence.
    let rot_y = t * 0.22;
    let rot_x = 0.35 + 0.25 * (t * 0.11).sin();
    let (cy_rot, sy_rot) = (rot_y.cos(), rot_y.sin());
    let (cx_rot, sx_rot) = (rot_x.cos(), rot_x.sin());

    let view_scale = 0.45;

    let mut grid = vec![0.0_f64; w * h];
    let dw = (w.saturating_sub(1)).max(1) as f64;
    let dh = (h.saturating_sub(1)).max(1) as f64;

    for row in 0..h {
      let py = (row as f64 / dh - 0.5) * 2.0;
      let base = row * w;
      for col in 0..w {
        let px = (col as f64 / dw - 0.5) * 2.0 * ax;

        // Step 1: screen pixel sits in the view plane. Treat (px, py) as
        // a point on the projected sphere. Recover its 3D position by
        // assuming it lies on the unit sphere with the visible hemisphere
        // facing the viewer.
        let r2 = px * px + py * py;
        if r2 > view_scale * view_scale {
          continue;
        }
        let nx = px / view_scale;
        let ny = py / view_scale;
        let nr2 = nx * nx + ny * ny;
        if nr2 > 1.0 {
          continue;
        }
        let nz = (1.0 - nr2).sqrt(); // visible hemisphere (z > 0)

        // Step 2: undo the 3D rotation to find the original sphere point
        // (the sphere is rotating; we look at it from a fixed viewer).
        // Inverse rotation: rotate by (-rot_x), then (-rot_y).
        let s1 = nx;
        let s2 = cx_rot * ny + sx_rot * nz;
        let s3 = -sx_rot * ny + cx_rot * nz;
        let q1 = cy_rot * s1 - sy_rot * s3;
        let q3 = sy_rot * s1 + cy_rot * s3;
        let q2 = s2;

        // Step 3: stereographically project (q1, q2, q3) on S² back to
        // the flat plane (X, Y) ∈ ℝ².
        let Some((plane_x, plane_y)) = stereographic(q1, q2, q3) else {
          continue;
        };

        // Step 4: pattern lives on the flat plane. Use a checkerboard
        // plus a fbm modulation so the "bending" of the grid onto the
        // sphere is obvious.
        let checker =
          if ((plane_x * 2.0).floor() + (plane_y * 2.0).floor()).rem_euclid(2.0) < 0.5 { 0.30 } else { 0.85 };
        let texture = fbm(plane_x * 1.5, plane_y * 1.5, 4);
        let edge = ((plane_x * 4.0).fract() - 0.5).abs().min(((plane_y * 4.0).fract() - 0.5).abs());
        let edge_glow = (-(edge - 0.05).powi(2) / 0.001).exp() * 0.45;

        // Sphere shading: brighter near the silhouette (rim light), dim
        // toward the centre.
        let shade = (1.0 - nz).powf(0.6) * 0.4 + 0.6;

        let value = (checker * 0.45 + texture * 0.20 + edge_glow) * shade;
        grid[base + col] = clamp(value * contrast);
      }
    }
    // Use inverse_stereographic somewhere visible -- keep it as a public
    // helper so the symmetric forward/inverse pair stays available for
    // future variants. Silence the unused warning here.
    let _ = inverse_stereographic(0.0, 0.0);
    render_field(ctx, &grid, TH, &STYLE_PLANE_BEND, out);
  }
}
