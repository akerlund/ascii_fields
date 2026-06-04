//! Möbius transformations on the Poincaré disk: warp a structured pattern
//! (concentric rings + radial spokes + fbm texture) by a time-varying
//! disk automorphism
//!
//!     T(z) = exp(i·θ) · (z - a) / (1 - ā·z)
//!
//! where the pole `a` orbits inside the unit disk and the rotation `θ` drifts
//! linearly. This is a one-parameter family of conformal maps that fix the
//! unit circle setwise; the visual effect is hyperbolic "panning" of the
//! disk's interior toward / away from the moving pole, with the rotation
//! adding a slow swirl.
//!
//! Rendering is inverse: for each screen pixel, apply T to obtain the
//! original coordinate, sample the pattern there, and the structure
//! appears to flow. Outside the unit disk a soft mask fades the field to
//! zero so the boundary of the geometry is visible without a hard edge.

use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, render_field, FieldStyle};
use crate::noise::fbm;

const STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "plasma" };
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

pub struct Mobius;

impl Animation for Mobius {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let w = ctx.width;
    let h = ctx.height;
    let ax = w as f64 / (h as f64 * 2.0).max(1.0);
    let contrast = ctx.options.contrast;
    let t = ctx.elapsed;

    // Pole orbits slowly inside the disk; rotation drifts linearly. Keeping
    // |a| well below 1 keeps the transformation tame and avoids extreme
    // numerical pinch at the unit circle.
    let a_radius = 0.55 + 0.15 * (t * 0.11).sin();
    let a_radius = a_radius * 0.55;
    let a_phase = t * 0.18;
    let ax_pole = a_radius * a_phase.cos();
    let ay_pole = a_radius * a_phase.sin();
    let theta = t * 0.42;
    let (ctheta, stheta) = (theta.cos(), theta.sin());

    let mut grid = vec![0.0_f64; w * h];
    let dw = (w.saturating_sub(1)).max(1) as f64;
    let dh = (h.saturating_sub(1)).max(1) as f64;

    for row in 0..h {
      let py = (row as f64 / dh - 0.5) * 2.0;
      let base = row * w;
      for col in 0..w {
        let px = (col as f64 / dw - 0.5) * 2.0 * ax;
        // Möbius: w = e^{iθ} · (z - a) / (1 - ā·z)
        //
        // For z = px + i*py and a = ax_pole + i*ay_pole:
        //   numerator   = (px - ax_pole) + i (py - ay_pole)
        //   ā·z         = (ax_pole·px + ay_pole·py)
        //                  + i (ax_pole·py - ay_pole·px)
        //   denominator = 1 - ā·z
        //               = (1 - ax_pole·px - ay_pole·py)
        //                  + i (ay_pole·px - ax_pole·py)
        let num_x = px - ax_pole;
        let num_y = py - ay_pole;
        let den_x = 1.0 - (ax_pole * px + ay_pole * py);
        let den_y = ay_pole * px - ax_pole * py;
        let denom = den_x * den_x + den_y * den_y + 1e-9;
        let zx = (num_x * den_x + num_y * den_y) / denom;
        let zy = (num_y * den_x - num_x * den_y) / denom;
        // Apply rotation by θ.
        let zx_rot = zx * ctheta - zy * stheta;
        let zy_rot = zx * stheta + zy * ctheta;

        // Pattern: concentric rings + radial spokes + fbm texture. The
        // rings collapse near the disk boundary under the Möbius map,
        // which is what makes the warp legible.
        let r = (zx_rot * zx_rot + zy_rot * zy_rot).sqrt();
        let phi = zy_rot.atan2(zx_rot);
        let rings = 0.5 + 0.5 * (r * 13.0).sin();
        let spokes = 0.5 + 0.5 * (phi * 6.0).cos();
        let texture = fbm(zx_rot * 3.0, zy_rot * 3.0, 4);
        let pattern = rings * 0.45 + spokes * 0.25 + texture * 0.40;

        // Disk mask: smoothstep fade just inside |z| = 1 so the boundary
        // is visible as a soft ring instead of a hard cut.
        let pr = (px * px + py * py).sqrt();
        let mask = (1.0 - (pr - 0.92) / 0.10).clamp(0.0, 1.0);

        grid[base + col] = clamp(pattern * mask * 1.45 * contrast);
      }
    }
    render_field(ctx, &grid, TH, &STYLE, out);
  }
}
