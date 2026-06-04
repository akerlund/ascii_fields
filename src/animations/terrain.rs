use rand::{Rng, SeedableRng};
use rand_pcg::Pcg32;

use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, render_field, FieldStyle};
use crate::noise::fbm_seeded;

use super::field_common::{aspect, dims, pulse, FrameScratch, FIELD_TH, LINE_TH};

const SEISMO_STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "geologic" };

/// A single seismic event. Magnitude controls how much the local elevation
/// gets pushed up by the wave, speed controls how fast the wavefront moves.
#[derive(Clone, Copy)]
struct Quake {
  x: f64,
  y: f64,
  t0: f64,
  magnitude: f64,
  speed: f64,
}

pub struct Seismograph {
  scratch: FrameScratch,
  quakes: Vec<Quake>,
  next_spawn: f64,
  last_elapsed: f64,
  rng: Pcg32,
}

impl Default for Seismograph {
  fn default() -> Self {
    Self {
      scratch: FrameScratch::default(),
      quakes: Vec::new(),
      next_spawn: 0.0,
      last_elapsed: 0.0,
      rng: Pcg32::seed_from_u64(0xEA47_4A41_5345_4953),
    }
  }
}

/// How long a quake's ring stays visible (after this its magnitude has decayed
/// to a negligible level so we drop it from the active list).
const QUAKE_LIFETIME: f64 = 14.0;

impl Animation for Seismograph {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    // Time-rewind reset (export, HUD scrub).
    if ctx.elapsed < self.last_elapsed {
      self.quakes.clear();
      self.next_spawn = 0.0;
    }
    self.last_elapsed = ctx.elapsed;

    // Spawn new quakes at jittered intervals until we have caught up.
    while ctx.elapsed >= self.next_spawn {
      // Magnitude distribution: square a uniform [0, 1] so most quakes are
      // small and the occasional one is large (like real magnitude curves).
      let m: f64 = self.rng.gen_range(0.35_f64..1.0_f64);
      self.quakes.push(Quake {
        x: self.rng.gen_range(0.10..0.90),
        y: self.rng.gen_range(0.10..0.90),
        t0: self.next_spawn,
        magnitude: m * m * 1.4,
        speed: self.rng.gen_range(0.22..0.45),
      });
      // Inter-arrival time: short for the next aftershock, sometimes long.
      self.next_spawn += self.rng.gen_range(1.2..4.0);
    }

    self.quakes.retain(|q| ctx.elapsed - q.t0 < QUAKE_LIFETIME);

    let (w, h, dw, dh) = dims(ctx);
    let ax = aspect(ctx);
    let t = ctx.elapsed;
    // Slow drift of the base topography so the landscape itself is alive
    // even between quakes.
    let drift = t * 0.06;

    let grid = self.scratch.grid(w * h);
    for row in 0..h {
      let v = row as f64 / dh;
      let base = row * w;
      for col in 0..w {
        let u = col as f64 / dw;

        // Base topography: smooth fbm gives mountains and valleys.
        let elev_base = fbm_seeded(u * 3.5 + drift, v * 3.5 - drift * 0.5, 5, 12345);

        // Quake contribution: each active quake adds a Gaussian-pinched
        // ring centred on its current wavefront. Multiple ongoing events
        // produce overlapping ripples in the contour pattern.
        let mut wave = 0.0_f64;
        for q in &self.quakes {
          let age = t - q.t0;
          if age <= 0.0 {
            continue;
          }
          let dx = (u - q.x) * ax;
          let dy = v - q.y;
          let r = (dx * dx + dy * dy).sqrt();
          let front = q.speed * age;
          // Wavefront grows wider as it propagates (energy spreads out).
          let width = 0.020 + 0.005 * age;
          let ring_offset = r - front;
          let ring = (-(ring_offset * ring_offset) / (width * width)).exp();
          // Amplitude decay: spatial 1/r falloff plus temporal exp.
          let temporal = (-age * 0.18).exp();
          let spatial = 1.0 / (1.0 + 4.0 * r);
          wave += q.magnitude * ring * temporal * spatial;
        }

        // Combined elevation. Waves push the elevation up locally, so the
        // contour lines bunch up at the wavefront -- the visual signature
        // of seismic deformation on a topo map.
        let elev = elev_base + wave * 0.45;

        // Contour lines at constant elevation intervals.
        let contour = pulse(((elev * 16.0).fract() - 0.5).abs(), 0.0030);
        // Faint shade from elevation itself so the topography reads even
        // where no contour line passes through the pixel.
        let shade = elev_base * 0.18;
        // Wave brightness as a glow on top of the contours so the
        // wavefronts are also directly visible.
        let glow = wave * 0.18;

        grid[base + col] = clamp((contour + shade + glow) * ctx.options.contrast);
      }
    }
    render_field(ctx, grid, LINE_TH, &SEISMO_STYLE, out);
  }
}

const CAUSTICS_STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "bathymetry" };
#[derive(Default)]
pub struct Caustics {
  scratch: FrameScratch,
}
impl Animation for Caustics {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let (w, h, dw, dh) = dims(ctx);
    let t = ctx.elapsed * 0.55;
    let grid = self.scratch.grid(w * h);
    for row in 0..h {
      let v = row as f64 / dh;
      let base = row * w;
      for col in 0..w {
        let u = col as f64 / dw;
        let a = (u * 14.0 + (v * 8.0 + t).sin() * 1.6 + t).sin();
        let b = (v * 12.0 + (u * 10.0 - t * 0.8).cos() * 1.4 - t * 0.7).sin();
        let c = ((u + v) * 10.0 + t * 0.5).sin();
        let value = pulse(a, 0.018) + pulse(b, 0.018) + 0.6 * pulse(c, 0.014);
        grid[base + col] = clamp(value * 0.65 * ctx.options.contrast);
      }
    }
    render_field(ctx, grid, LINE_TH, &CAUSTICS_STYLE, out);
  }
}

const DUNES_STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "dusk" };
#[derive(Default)]
pub struct Dunes {
  scratch: FrameScratch,
}
impl Animation for Dunes {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let (w, h, dw, dh) = dims(ctx);
    let t = ctx.elapsed * 0.18;
    let grid = self.scratch.grid(w * h);
    for row in 0..h {
      let v = row as f64 / dh;
      let base = row * w;
      for col in 0..w {
        let u = col as f64 / dw;
        let slope = v + 0.08 * (u * 5.0 + t).sin();
        let ripples =
          ((u * 34.0 + slope * 14.0 - t * 5.0 + fbm_seeded(u * 4.0, v * 4.0, 3, 23) * 3.0).sin() * 0.5 + 0.5)
            .powf(3.0);
        let ridge = pulse(ripples - 0.92, 0.010);
        let shade = 0.28 + 0.45 * v + 0.35 * ridge;
        grid[base + col] = clamp(shade * ctx.options.contrast);
      }
    }
    render_field(ctx, grid, FIELD_TH, &DUNES_STYLE, out);
  }
}

const TOPO_STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "geologic" };
#[derive(Default)]
pub struct Topography {
  scratch: FrameScratch,
}
impl Animation for Topography {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let (w, h, dw, dh) = dims(ctx);
    let t = ctx.elapsed * 0.12;
    let grid = self.scratch.grid(w * h);
    for row in 0..h {
      let v = row as f64 / dh;
      let base = row * w;
      for col in 0..w {
        let u = col as f64 / dw;
        let elev = fbm_seeded(u * 4.2 + t, v * 4.2 - t * 0.5, 5, 771);
        let contour = pulse(((elev * 15.0).fract() - 0.5).abs(), 0.0025);
        let river = pulse(elev - (0.43 + 0.04 * (u * 8.0 + t * 4.0).sin()), 0.0018)
          * (0.6 + 0.4 * (v * 10.0).sin().abs());
        grid[base + col] = clamp((contour * 0.85 + river * 0.75 + elev * 0.20) * ctx.options.contrast);
      }
    }
    render_field(ctx, grid, LINE_TH, &TOPO_STYLE, out);
  }
}
