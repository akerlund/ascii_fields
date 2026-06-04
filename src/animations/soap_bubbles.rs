//! Soap bubbles drifting upward with iridescent rims. Each bubble has its own
//! grow / live / pop envelope and a per-bubble iridescence phase so they shift
//! through the colour palette differently. The rim brightness is modulated by
//! a sine of (angle, radial-position) so bands of colour sweep around each
//! bubble -- the trick that turns the brightness->colour palette mapping into
//! something that *reads* as a thin-film rainbow.
//!
//! State is per-bubble in a `Vec<Bubble>`; the spawn schedule advances by a
//! jittered interval, capped at MAX_BUBBLES. Same `last_elapsed` time-rewind
//! reset as `drops` so exports and HUD scrubs do not leave stale bubbles.

use std::f64::consts::TAU;

use rand::{Rng, SeedableRng};
use rand_pcg::Pcg32;

use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, render_field, FieldStyle};

const STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "spectrum" };
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

const SPAWN_DT_BASE: f64 = 1.4;
const LIFETIME_MIN: f64 = 6.0;
const LIFETIME_MAX: f64 = 12.0;
const GROW_DURATION: f64 = 1.0;
const POP_DURATION: f64 = 0.45;
const MAX_BUBBLES: usize = 16;

/// Smooth time-varying horizontal displacement representing integrated wind.
/// Sum of three sinusoids at different rates so the wind never quite repeats
/// and the bubbles see real drift instead of a clean oscillation.
#[inline]
fn wind_displacement(t: f64) -> f64 {
  0.16 * (t * 0.07).sin() + 0.07 * (t * 0.19 + 1.7).sin() + 0.04 * (t * 0.43 + 3.1).sin()
}

#[derive(Clone, Copy)]
struct Bubble {
  x: f64,
  y: f64,
  vx: f64,          // horizontal drift
  vy: f64,          // vertical drift (negative = rising)
  r_target: f64,    // radius once fully grown
  t0: f64,          // spawn time
  lifetime: f64,    // total seconds before pop completes
  phase_seed: f64,  // per-bubble iridescence phase
  wobble_amp: f64,  // sideways wobble amplitude
  wobble_freq: f64, // wobble frequency in Hz
  tone_offset: f64, // per-bubble brightness offset -> palette colour shift
}

pub struct SoapBubbles {
  bubbles: Vec<Bubble>,
  next_spawn: f64,
  last_elapsed: f64,
  rng: Pcg32,
}

impl Default for SoapBubbles {
  fn default() -> Self {
    Self { bubbles: Vec::new(), next_spawn: 0.0, last_elapsed: 0.0, rng: Pcg32::from_entropy() }
  }
}

impl Animation for SoapBubbles {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    // Time-rewind reset (export, HUD scrub).
    if ctx.elapsed < self.last_elapsed {
      self.bubbles.clear();
      self.next_spawn = 0.0;
    }
    self.last_elapsed = ctx.elapsed;
    let ax = ctx.width as f64 / (ctx.height as f64 * 2.0).max(1.0);

    // Spawn new bubbles at jittered intervals until we have caught up to now.
    while ctx.elapsed >= self.next_spawn {
      let r_target: f64 = self.rng.gen_range(0.05..0.16);
      // Strong size -> speed coupling: tiny bubbles whoosh up, big ones
      // lumber. Cubic scaling (0.05^3 / r^3) makes the difference
      // visually obvious instead of subtle.
      let size_speed = (0.07_f64 / r_target.max(0.04)).powi(2).clamp(0.5, 4.5);
      self.bubbles.push(Bubble {
        x: self.rng.gen_range(0.10..0.90),
        y: self.rng.gen_range(0.95..1.10),
        vx: self.rng.gen_range(-0.010..0.010),
        // Base rise speed bumped from 0.025..0.055 to 0.060..0.110 so even
        // big bubbles cross the screen on a reasonable timescale.
        vy: -self.rng.gen_range(0.060..0.110) * size_speed,
        r_target,
        t0: self.next_spawn,
        lifetime: self.rng.gen_range(LIFETIME_MIN..LIFETIME_MAX),
        phase_seed: self.rng.gen_range(0.0..TAU),
        wobble_amp: self.rng.gen_range(0.005..0.018),
        wobble_freq: self.rng.gen_range(0.7..1.6),
        // Brightness offset shifts each bubble into a different region of
        // the spectrum palette so bubbles look like a multi-colour mix
        // rather than 16 copies of the same rainbow.
        tone_offset: self.rng.gen_range(-0.18..0.18),
      });
      self.next_spawn += SPAWN_DT_BASE * self.rng.gen_range(0.6..1.4);
    }

    self.bubbles.retain(|b| ctx.elapsed - b.t0 < b.lifetime);
    while self.bubbles.len() > MAX_BUBBLES {
      self.bubbles.remove(0);
    }

    // Global wind: a smooth time-varying horizontal drift that pushes all
    // bubbles, weighted by their size (smaller bubbles get pushed harder
    // because they have less momentum). The wind itself is a sum of three
    // sinusoids at different rates so it never repeats cleanly.
    let wind_phase = wind_displacement(ctx.elapsed);

    // Precompute the per-bubble live geometry once, so the inner cell loop is
    // a tight sum over a small Vec.
    let visible: Vec<(f64, f64, f64, f64, f64)> = self
      .bubbles
      .iter()
      .filter_map(|b| {
        let age = ctx.elapsed - b.t0;
        // Wind susceptibility: smaller bubbles drift more in the wind.
        let wind_susceptibility = (0.08_f64 / b.r_target.max(0.04)).clamp(0.4, 3.0);
        let wind_offset = (wind_phase - wind_displacement(b.t0)) * wind_susceptibility;
        let bx = b.x + b.vx * age + b.wobble_amp * (age * b.wobble_freq * TAU).sin() + wind_offset;
        let by = b.y + b.vy * age;
        // Envelope: ease-out grow -> hold -> ease-in pop
        let r = if age < GROW_DURATION {
          let t = age / GROW_DURATION;
          b.r_target * (1.0 - (1.0 - t).powi(3))
        } else if age < b.lifetime - POP_DURATION {
          b.r_target
        } else {
          let t = ((b.lifetime - age) / POP_DURATION).max(0.0);
          b.r_target * t * t
        };
        if r < 1e-3 || !(-0.25..1.25).contains(&by) {
          return None;
        }
        Some((bx, by, r, b.phase_seed, b.tone_offset))
      })
      .collect();

    let contrast = ctx.options.contrast;
    let mut grid = vec![0.0_f64; ctx.width * ctx.height];
    let dw = (ctx.width.saturating_sub(1)).max(1) as f64;
    let dh = (ctx.height.saturating_sub(1)).max(1) as f64;

    for row in 0..ctx.height {
      let v = row as f64 / dh;
      let base = row * ctx.width;
      for col in 0..ctx.width {
        let u = col as f64 / dw;
        let mut value = 0.0_f64;
        for &(bx, by, r, phase, tone) in &visible {
          let dx = (u - bx) * ax;
          let dy = v - by;
          let dist = (dx * dx + dy * dy).sqrt();
          // Pre-cull anything outside the bubble + its outer halo.
          if dist > r * 1.30 {
            continue;
          }
          let radial = (dist / r).min(1.3);
          // Rim: Gaussian-pinched ring just inside the surface.
          let rim_radius = r * 0.94;
          let rim_sigma = (r * 0.07).max(0.006);
          let rim = (-((dist - rim_radius).powi(2)) / (rim_sigma * rim_sigma)).exp();
          // Halo: wider, dimmer ring just outside -- gives bubbles a soft
          // bloom against dark backgrounds.
          let halo_sigma = (r * 0.18).max(0.015);
          let halo = (-((dist - r * 1.05).powi(2)) / (halo_sigma * halo_sigma)).exp() * 0.22;
          // Two iridescence bands at different frequencies + phases. The
          // brightness sweep paints colour through the palette; layering
          // makes the bands feel less like a single rotating sinusoid.
          let angle = dy.atan2(dx);
          let band_a = 0.5 + 0.5 * (angle * 3.0 + radial * 7.0 + phase).sin();
          let band_b = 0.5 + 0.5 * (angle * 5.0 - radial * 4.0 + phase * 1.7).cos();
          let iridescence = 0.7 * band_a + 0.3 * band_b;
          // Faint inner volume so the centre is not pure black.
          let interior = if dist < r { (1.0 - radial.powi(2)) * 0.18 } else { 0.0 };
          // Specular highlight: bright Gaussian spot offset toward the
          // upper-left (light source convention). Only renders inside the
          // bubble silhouette.
          let spec_dx = dx - (-r * 0.40);
          let spec_dy = dy - (-r * 0.40);
          let spec_sigma = (r * 0.18).max(0.012);
          let spec_gauss = (-(spec_dx * spec_dx + spec_dy * spec_dy) / (spec_sigma * spec_sigma)).exp();
          let specular = if dist < r { spec_gauss * 0.55 } else { 0.0 };

          // tone shifts the bubble's average brightness, biasing where in
          // the palette the iridescent rim peaks. The shift is clamped to
          // stay inside the displayable [0, 1] range.
          let contribution = (rim * (0.40 + 0.55 * iridescence) + halo + interior + specular + tone).max(0.0);
          value = value.max(contribution);
        }
        grid[base + col] = clamp(value * contrast);
      }
    }
    render_field(ctx, &grid, TH, &STYLE, out);
  }
}
