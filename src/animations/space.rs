use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, density_char, render_field, render_glyph_field, FieldStyle};
use crate::noise::fbm_seeded;

use super::field_common::{aspect, dims, pulse, put, FrameScratch, FIELD_TH, LINE_TH};

const NBODY_STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "nebula" };
const STELLAR_STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "stellar" };

#[derive(Default)]
pub struct NBody {
  scratch: FrameScratch,
}
impl Animation for NBody {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let (w, h, dw, dh) = dims(ctx);
    let ax = aspect(ctx);
    let t = ctx.elapsed * 0.33;
    let bodies = [
      (0.50 + 0.23 * t.cos(), 0.50 + 0.20 * t.sin(), 1.0),
      (0.50 + 0.16 * (t * 1.7 + 2.1).cos(), 0.50 + 0.26 * (t * 1.7 + 2.1).sin(), 0.72),
      (0.50 + 0.32 * (t * 0.73 + 4.0).cos(), 0.50 + 0.10 * (t * 0.73 + 4.0).sin(), 0.55),
      (0.50 + 0.10 * (t * 2.3 + 1.0).cos(), 0.50 + 0.34 * (t * 2.3 + 1.0).sin(), 0.35),
    ];
    let (grid, glyphs) = self.scratch.grid_and_glyphs(w * h);
    for row in 0..h {
      let v = row as f64 / dh;
      for col in 0..w {
        let u = col as f64 / dw;
        let mut value = 0.0;
        for &(bx, by, mass) in &bodies {
          let dx = (u - bx) * ax;
          let dy = v - by;
          value += mass * 0.018 / (dx * dx + dy * dy + 0.002);
        }
        grid[row * w + col] = clamp(value * ctx.options.contrast);
      }
    }
    for (idx, glyph) in glyphs.iter_mut().enumerate() {
      *glyph = density_char(grid[idx], FIELD_TH);
    }
    for &(bx, by, mass) in &bodies {
      put(
        grid,
        glyphs,
        w,
        h,
        (bx * dw).round() as i64,
        (by * dh).round() as i64,
        1.0,
        if mass > 0.7 { '@' } else { '*' },
      );
    }
    render_glyph_field(ctx, grid, glyphs, &NBODY_STYLE, out);
  }
}

// Supernova has two parameters tuned so successive bursts overlap:
//   SPAWN_INTERVAL = how often a new one is born
//   LIFETIME       = how long each one is visible
// LIFETIME > SPAWN_INTERVAL gives an overlap window where the previous burst
// is still fading while the next is just igniting.
const SUPERNOVA_SPAWN_INTERVAL: f64 = 4.0;
const SUPERNOVA_LIFETIME: f64 = 5.5;

fn supernova_seed(epoch: i64) -> i64 {
  // Cheap integer hash: Knuth multiplier + odd-prime add. Each epoch gets a
  // visually distinct filament pattern from the same fbm.
  epoch.wrapping_mul(2_654_435_761).wrapping_add(13)
}

fn supernova_center(epoch: i64) -> (f64, f64) {
  // Deterministic per-epoch offset, biased near screen centre so the burst
  // stays mostly on-screen.
  let h = supernova_seed(epoch) as u64;
  let rx = ((h & 0xFFFF) as f64) / 65535.0;
  let ry = (((h >> 16) & 0xFFFF) as f64) / 65535.0;
  (0.35 + 0.30 * rx, 0.35 + 0.30 * ry)
}

#[inline]
fn supernova_contribution(u: f64, v: f64, ax: f64, age: f64, seed: i64, cx: f64, cy: f64) -> f64 {
  if !(0.0..1.0).contains(&age) {
    return 0.0;
  }
  let radius = 0.04 + age * 0.62;
  let dx = (u - cx) * ax;
  let dy = v - cy;
  let r = (dx * dx + dy * dy).sqrt();
  let a = dy.atan2(dx);
  let filament = 0.7 + 0.3 * (a * 13.0 + fbm_seeded(u * 5.0, v * 5.0, 3, seed) * 6.0).sin().abs();
  let shell = pulse(r - radius * filament, 0.0012 + age * 0.003) * (1.0 - age).powf(0.35);
  let core = pulse(r, 0.008 + age * 0.050) * (1.0 - age);
  shell + core
}

#[derive(Default)]
pub struct Supernova {
  scratch: FrameScratch,
}
impl Animation for Supernova {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let (w, h, dw, dh) = dims(ctx);
    let ax = aspect(ctx);
    // Which supernovae overlap "now"? Each lasts LIFETIME; new ones spawn
    // every SPAWN_INTERVAL. Since LIFETIME < 2 * SPAWN_INTERVAL, at most two
    // bursts are alive simultaneously -- the previous (fading) and the
    // current (just born).
    let current_epoch = (ctx.elapsed / SUPERNOVA_SPAWN_INTERVAL).floor() as i64;
    let bursts: [(i64, f64); 2] = [
      (current_epoch, ctx.elapsed - current_epoch as f64 * SUPERNOVA_SPAWN_INTERVAL),
      (current_epoch - 1, ctx.elapsed - (current_epoch - 1) as f64 * SUPERNOVA_SPAWN_INTERVAL),
    ];
    // Precompute per-burst params so the inner loop just sums contributions.
    let burst_params: Vec<(f64, i64, f64, f64)> = bursts
      .iter()
      .filter_map(|&(epoch, age_seconds)| {
        if epoch < 0 {
          return None;
        }
        let age = age_seconds / SUPERNOVA_LIFETIME;
        if !(0.0..1.0).contains(&age) {
          return None;
        }
        let (cx, cy) = supernova_center(epoch);
        Some((age, supernova_seed(epoch), cx, cy))
      })
      .collect();

    let grid = self.scratch.grid(w * h);
    for row in 0..h {
      let v = row as f64 / dh;
      let base = row * w;
      for col in 0..w {
        let u = col as f64 / dw;
        let mut value = 0.0;
        for &(age, seed, cx, cy) in &burst_params {
          value += supernova_contribution(u, v, ax, age, seed, cx, cy);
        }
        grid[base + col] = clamp(value * ctx.options.contrast);
      }
    }
    render_field(ctx, grid, FIELD_TH, &STELLAR_STYLE, out);
  }
}

const SOLAR_WIND_STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "aurora" };

/// Solar wind redesigned as a particle simulation. Sun on the left emits
/// charged particles that stream right; a planet's magnetosphere in the
/// centre-right deflects them around a bow shock and into a downstream
/// magnetotail. Replaces the previous "two halves of sines + a circle"
/// rendering which did not look like solar wind to a viewer.
const SOLARWIND_EARTH_X: f64 = 0.66;
const SOLARWIND_EARTH_Y: f64 = 0.50;
const SOLARWIND_EARTH_RADIUS: f64 = 0.045;
const SOLARWIND_MAGNETOPAUSE: f64 = 0.13;
const SOLARWIND_N_PARTICLES: usize = 500;

#[derive(Clone, Copy)]
struct SolarParticle {
  x: f64,
  y: f64,
  vx: f64,
  vy: f64,
}

pub struct SolarWind {
  particles: Vec<SolarParticle>,
  last: f64,
  rng: rand_pcg::Pcg32,
}

impl Default for SolarWind {
  fn default() -> Self {
    use rand::SeedableRng;
    Self { particles: Vec::new(), last: 0.0, rng: rand_pcg::Pcg32::seed_from_u64(0x501A_271E_D000) }
  }
}

impl SolarWind {
  fn fresh_particle(&mut self) -> SolarParticle {
    use rand::Rng;
    SolarParticle {
      x: self.rng.gen_range(-0.05..0.02),
      y: self.rng.gen_range(0.05..0.95),
      vx: self.rng.gen_range(0.30..0.45),
      vy: self.rng.gen_range(-0.010..0.010),
    }
  }
}

impl Animation for SolarWind {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    if ctx.elapsed < self.last {
      self.particles.clear();
    }
    let dt = (ctx.elapsed - self.last).clamp(0.0, 0.1);
    self.last = ctx.elapsed;

    // Top up the particle population.
    while self.particles.len() < SOLARWIND_N_PARTICLES {
      let p = self.fresh_particle();
      self.particles.push(p);
    }

    // Update each particle. Force model:
    //   * Repulsion from the planet ∝ 1/r^3 so the bow shock has a sharp
    //     standoff distance but the field reaches well upstream.
    //   * Light damping to keep things stable.
    //   * Forward drift restoration when far from the planet so particles
    //     do not stall after deflection.
    let mut respawn: Vec<usize> = Vec::new();
    for (i, p) in self.particles.iter_mut().enumerate() {
      let dx = p.x - SOLARWIND_EARTH_X;
      let dy = p.y - SOLARWIND_EARTH_Y;
      let r2 = dx * dx + dy * dy;
      let r = r2.sqrt().max(SOLARWIND_EARTH_RADIUS * 0.5);
      let push = (0.0010 / r.powi(3)).min(5.0);
      p.vx += dx / r * push * dt * 60.0;
      p.vy += dy / r * push * dt * 60.0;

      p.vx *= 0.985;
      p.vy *= 0.985;
      // Restore forward drift when far from planet so the wind keeps blowing.
      if r > SOLARWIND_MAGNETOPAUSE * 1.5 {
        p.vx += (0.40 - p.vx) * 0.04;
      }

      p.x += p.vx * dt;
      p.y += p.vy * dt;

      let inside_planet = r < SOLARWIND_EARTH_RADIUS * 0.7;
      if p.x > 1.05 || !(-0.05..1.05).contains(&p.y) || inside_planet {
        respawn.push(i);
      }
    }
    for i in respawn {
      self.particles[i] = self.fresh_particle();
    }

    // Render.
    let (w, h, dw, dh) = dims(ctx);
    let ax = aspect(ctx);
    let mut grid = vec![0.0_f64; w * h];

    // Background sun glow on the left.
    for row in 0..h {
      let v = row as f64 / dh;
      let base = row * w;
      for col in 0..w {
        let u = col as f64 / dw;
        let dy = v - 0.50;
        let dx = u * ax;
        let glow = 0.55 * (-(dx * dx + dy * dy) / 0.06).exp();
        grid[base + col] = grid[base + col].max(glow);
      }
    }

    // Particles: each painted as a single bright pixel with brightness
    // proportional to speed, so the deflection arcs read as motion streaks
    // when many particles cluster along the same streamline.
    for p in &self.particles {
      let col = (p.x * dw).round() as i64;
      let row = (p.y * dh).round() as i64;
      if col < 0 || col >= w as i64 || row < 0 || row >= h as i64 {
        continue;
      }
      let speed = (p.vx * p.vx + p.vy * p.vy).sqrt();
      let bright = (0.40 + 1.4 * speed).clamp(0.30, 1.0);
      let idx = row as usize * w + col as usize;
      if grid[idx] < bright {
        grid[idx] = bright;
      }
    }

    // Planet: solid disc + bright rim (atmosphere band).
    for row in 0..h {
      let v = row as f64 / dh;
      let base = row * w;
      for col in 0..w {
        let u = col as f64 / dw;
        let dx = (u - SOLARWIND_EARTH_X) * ax;
        let dy = v - SOLARWIND_EARTH_Y;
        let r = (dx * dx + dy * dy).sqrt();
        if r < SOLARWIND_EARTH_RADIUS {
          let depth = 1.0 - r / SOLARWIND_EARTH_RADIUS;
          grid[base + col] = (0.55 + 0.35 * depth).max(grid[base + col]);
        } else if r < SOLARWIND_EARTH_RADIUS * 1.6 {
          let rim_d = (r - SOLARWIND_EARTH_RADIUS) / (SOLARWIND_EARTH_RADIUS * 0.6);
          let rim = (1.0 - rim_d).max(0.0) * 0.45;
          grid[base + col] = grid[base + col].max(rim);
        }
      }
    }

    let contrast = ctx.options.contrast;
    for v in grid.iter_mut() {
      *v = clamp(*v * contrast);
    }
    render_field(ctx, &grid, LINE_TH, &SOLAR_WIND_STYLE, out);
  }
}
