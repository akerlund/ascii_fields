//! Open ocean with rolling swells, seen from a low perspective angle.
//!
//! Distinct from the existing `waves` mode (top-down surf hitting a beach)
//! and `caustics` (light patterns on a flat surface). This is the open
//! water itself: a 3D heightfield projected through a tilted camera, so
//! the viewer sees swells receding to a horizon, with foam crests on the
//! taller wave peaks and shifting reflections everywhere.
//!
//! Pipeline:
//! * For each screen row, the row maps to a "distance from camera" along
//!   the water plane via perspective division. Far rows correspond to
//!   distant water; near rows to water just in front of the camera.
//! * Sample the wave heightfield at (world_x, world_z) using a sum of
//!   three travelling waves with different wavelengths and directions
//!   (the classic Gerstner-like superposition without the displacement).
//! * Convert height to brightness: bright on wave crests (sky reflection),
//!   darker in troughs (deeper water).
//! * Add foam: where the height gradient is steepest (crest), spike the
//!   brightness with fbm-noise-modulated whitecaps.
//! * Horizon line near the top fades to sky.

use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, render_field, FieldStyle};
use crate::noise::fbm;

const STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "bathymetry" };
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

/// Camera height above the water plane in arbitrary world units.
const CAMERA_HEIGHT: f64 = 1.6;
/// World-z distance at which the screen horizon sits (effectively
/// infinity for our purposes).
const HORIZON_DISTANCE: f64 = 80.0;
/// Vertical position of the horizon line, 0..1 from top.
const HORIZON_Y: f64 = 0.32;

pub struct Ocean;

impl Animation for Ocean {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let w = ctx.width;
    let h = ctx.height;
    let ax = w as f64 / (h as f64 * 2.0).max(1.0);
    let dw = (w.saturating_sub(1)).max(1) as f64;
    let dh = (h.saturating_sub(1)).max(1) as f64;
    let t = ctx.elapsed;

    // Three wave systems running in different directions at different speeds
    // and wavelengths. Their superposition is the heightfield. The constants
    // are picked to look like ocean swell, not a regular cosine grid.
    let wave1 = (0.40, 1.0, 0.65, t * 1.20); // (k_x, k_z, amplitude, phase)
    let wave2 = (-0.55, 0.50, 0.40, t * 0.95);
    let wave3 = (0.25, -0.85, 0.28, t * 1.55);

    let mut grid = vec![0.0_f64; w * h];
    for row in 0..h {
      let sy = row as f64 / dh;
      // Map screen-y to camera-relative direction. Above the horizon is
      // sky; below is water.
      let base = row * w;
      if sy < HORIZON_Y {
        // Sky band: soft gradient.
        let sky = 0.18 - 0.10 * (HORIZON_Y - sy) / HORIZON_Y;
        for col in 0..w {
          grid[base + col] = clamp(sky.max(0.0));
        }
        continue;
      }

      // Compute world-z (distance from camera) for this screen row using
      // a simple perspective relation: y_screen = CAMERA_HEIGHT / z
      // gives z = CAMERA_HEIGHT / y_screen. Below-horizon offset:
      let screen_below = (sy - HORIZON_Y) / (1.0 - HORIZON_Y);
      // Avoid division-by-zero at the horizon line itself.
      let world_z = (CAMERA_HEIGHT / screen_below.max(1e-3)).min(HORIZON_DISTANCE);

      for col in 0..w {
        let su = col as f64 / dw;
        // Map screen-x to world-x at this distance. Closer rows have a
        // wider field of view in world-x, far rows have a narrower one.
        let world_x = (su - 0.5) * world_z * ax * 1.4;

        // Sum the three wave systems.
        let phase1 = wave1.0 * world_x + wave1.1 * world_z + wave1.3;
        let phase2 = wave2.0 * world_x + wave2.1 * world_z + wave2.3;
        let phase3 = wave3.0 * world_x + wave3.1 * world_z + wave3.3;
        let height = wave1.2 * phase1.sin() + wave2.2 * phase2.sin() + wave3.2 * phase3.sin();

        // Brightness from height: peaks bright (sky reflection), troughs
        // dim (deep water).
        let mut value = 0.40 + 0.30 * height;

        // Foam: where the height is near its local maximum and a fbm
        // noise term gates the foam patches.
        let foam_gate = (height - 0.55).max(0.0);
        let foam_noise = fbm(world_x * 1.6 + t * 0.5, world_z * 1.6, 3);
        let foam = foam_gate * (foam_noise - 0.3).max(0.0) * 1.6;
        value += foam;

        // Distance fade: very distant water dims toward the horizon's sky.
        let distance_fade = (world_z / HORIZON_DISTANCE).clamp(0.0, 1.0);
        value = value * (1.0 - 0.35 * distance_fade) + 0.10 * distance_fade;

        grid[base + col] = clamp(value * ctx.options.contrast);
      }
    }
    render_field(ctx, &grid, TH, &STYLE, out);
  }
}
