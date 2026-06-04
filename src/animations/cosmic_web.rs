//! Cosmic web: the large-scale filamentary structure of the universe, as if
//! you have zoomed out so far that individual galaxies have collapsed into
//! points strung along the boundaries of immense voids.
//!
//! Built from three noise layers stacked on top of each other:
//!
//! 1. A slow macro-density (clusters cluster).
//! 2. A ridge-shaped mid-frequency noise (`1 - |2*fbm - 1|`) producing the
//!    sharp filaments that connect dense regions.
//! 3. A high-frequency micro-noise that picks out individual galaxy points
//!    along the filaments and at the cluster nodes.
//!
//! The result is gated aggressively into a dark background so voids stay
//! black, then a gentle parallax drift keeps the structure moving without
//! obvious looping. Slight per-frame zoom breathing adds a subtle pulse.

use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, render_field, FieldStyle};
use crate::noise::{fbm, fbm_seeded};

const STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "stellar" };
const TH: &[(f64, char)] = &[
  (0.06, ' '),
  (0.15, '.'),
  (0.26, ':'),
  (0.40, '-'),
  (0.55, '='),
  (0.70, '+'),
  (0.83, '*'),
  (0.92, '#'),
  (1.01, '@'),
];

pub struct CosmicWeb;

impl Animation for CosmicWeb {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let w = ctx.width;
    let h = ctx.height;
    let ax = w as f64 / (h as f64 * 2.0).max(1.0);
    let contrast = ctx.options.contrast;
    let scale = 2.4 * ctx.options.scale.max(0.4);

    // Slow drift + breathing zoom for that "adrift in the void" feeling.
    let drift_x = ctx.elapsed * 0.014;
    let drift_y = ctx.elapsed * 0.009;
    let zoom = 1.0 + 0.04 * (ctx.elapsed * 0.05).sin();

    let mut grid = vec![0.0_f64; w * h];
    let dw = (w.saturating_sub(1)).max(1) as f64;
    let dh = (h.saturating_sub(1)).max(1) as f64;

    for row in 0..h {
      let v = row as f64 / dh;
      let base = row * w;
      for col in 0..w {
        let u = col as f64 / dw;
        let sx = ((u - 0.5) * ax + drift_x) * scale / zoom;
        let sy = ((v - 0.5) + drift_y) * scale / zoom;

        // Macro density: where the universe is denser overall. Drives whether
        // a region is "in a supercluster" or "in a void".
        let macro_density = fbm(sx * 0.45, sy * 0.45, 4);

        // Ridge noise: turns smooth fbm into sharp filaments by mirroring it
        // around 0.5. Powering the result makes the ridges narrower.
        let ridge_raw = fbm_seeded(sx * 1.4 + 7.7, sy * 1.4 + 3.3, 4, 1031);
        let ridge = 1.0 - (ridge_raw * 2.0 - 1.0).abs();
        let filament = ridge.powi(3) * (macro_density - 0.32).max(0.0) * 2.6;

        // Micro-noise: picks out individual galaxies along the filaments.
        let micro = fbm_seeded(sx * 6.0 + 11.1, sy * 6.0 - 2.5, 3, 4099);
        let galaxy = (filament * 1.4 + 0.05) * (micro - 0.55).max(0.0) * 5.5;

        // Node: bright knot at cluster centres where both macro and ridge
        // peak. Squared so they read as compact bright points.
        let node_macro = (macro_density - 0.62).max(0.0);
        let node = node_macro * node_macro * ridge.powi(2) * 6.0;

        // Combine. Gate the background hard so most pixels stay black.
        let level = (filament * 0.45 + galaxy + node).powf(1.35);
        grid[base + col] = clamp(level * contrast);
      }
    }
    render_field(ctx, &grid, TH, &STYLE, out);
  }
}
