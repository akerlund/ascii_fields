//! Wax lamp with real thermal physics. Old version: four hard-coded blobs
//! rising on fixed schedules. User wanted blobs that cool as they rise, fall
//! back down, and occasionally merge or split.
//!
//! Each blob carries:
//!   * position (x, y) in lamp coordinates
//!   * velocity (vx, vy)
//!   * mass (controls radius via cube root)
//!   * temperature in [0, 1], where 0 = ambient and 1 = at the heater
//!
//! Physics per frame:
//!   * Buoyancy force ∝ (temp - 0.45) -- hot rises, cold sinks.
//!   * Light drag damps both axes.
//!   * Small horizontal Brownian sway.
//!   * Temperature drains at a constant rate; close to the bottom (heater
//!     zone) it gets replenished, so a cold blob that sinks gets reheated
//!     and starts to rise again.
//!   * Lamp-shaped walls bounce blobs with a coefficient of restitution.
//!
//! Lifecycle:
//!   * Merging: any pair within 0.7 (r1+r2) coalesces, combining mass /
//!     momentum / temperature by mass-weighted average.
//!   * Splitting: blobs whose mass exceeds the split threshold split with
//!     low probability per frame; daughter gets a fraction of the mass and
//!     a kick in a random direction.
//!   * Spawning: if the population drops below a floor, small fresh blobs
//!     are introduced at the heater.
//!
//! The result is a continuously stirring lamp where blob populations evolve
//! organically -- no two runs look the same.

use std::f64::consts::TAU;

use rand::{Rng, SeedableRng};
use rand_pcg::Pcg32;

use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, render_field, FieldStyle};

const STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "lava" };
const TH: &[(f64, char)] = &[
  (0.08, ' '),
  (0.18, '.'),
  (0.30, ':'),
  (0.42, '-'),
  (0.54, '='),
  (0.66, '+'),
  (0.78, '*'),
  (0.90, '#'),
  (1.01, '@'),
];

const MAX_BLOBS: usize = 10;
const MIN_BLOBS: usize = 3;
const SPLIT_THRESHOLD: f64 = 1.4;
const HEATER_Y: f64 = 0.85; // y > this is in the heater zone

#[derive(Clone, Copy)]
struct Blob {
  x: f64,
  y: f64,
  vx: f64,
  vy: f64,
  mass: f64,
  temp: f64,
}

impl Blob {
  #[inline]
  fn radius(&self) -> f64 {
    0.06 * self.mass.cbrt()
  }
}

pub struct VaxLamp {
  blobs: Vec<Blob>,
  last: f64,
  rng: Pcg32,
}

impl Default for VaxLamp {
  fn default() -> Self {
    Self { blobs: Vec::new(), last: 0.0, rng: Pcg32::seed_from_u64(0xABAD_D00D_BEEF) }
  }
}

impl VaxLamp {
  fn lamp_width(&self, y: f64) -> f64 {
    // Lamp profile: pinched at top and bottom, fattest in the middle.
    let bell = (1.0 - (y - 0.5).powi(2) * 4.0).clamp(0.05, 1.0);
    0.13 + 0.08 * bell
  }

  fn spawn_fresh(&mut self) {
    let x = self.rng.gen_range(0.35..0.65);
    let y = 0.92;
    self.blobs.push(Blob {
      x,
      y,
      vx: self.rng.gen_range(-0.02..0.02),
      vy: 0.0,
      mass: self.rng.gen_range(0.30..0.60),
      temp: self.rng.gen_range(0.85..1.00),
    });
  }
}

impl Animation for VaxLamp {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    if ctx.elapsed < self.last {
      self.blobs.clear();
    }
    let dt = (ctx.elapsed - self.last).clamp(0.0, 0.1);
    self.last = ctx.elapsed;

    // Seed initial population.
    while self.blobs.len() < MIN_BLOBS {
      self.spawn_fresh();
    }

    // Pre-compute lamp widths for current blob positions to use in collision
    // resolution below; capture by index to avoid borrow conflicts.
    let widths: Vec<f64> = self.blobs.iter().map(|b| self.lamp_width(b.y)).collect();

    // Physics integration.
    for (i, b) in self.blobs.iter_mut().enumerate() {
      // Buoyancy: positive force pushes upward (negative y in screen-down
      // convention). Hot up, cool down.
      let buoyancy = -(b.temp - 0.45) * 0.45;
      b.vy += buoyancy * dt;

      // Brownian horizontal jiggle.
      let jiggle: f64 = self.rng.gen_range(-0.04..0.04);
      b.vx += jiggle * dt;

      // Drag.
      b.vx *= 1.0 - 0.6 * dt;
      b.vy *= 1.0 - 0.6 * dt;

      // Temperature: continuous cooling, with replenishment if close to
      // the heater at the bottom.
      let cooling = 0.25 * dt;
      let heating = if b.y > HEATER_Y {
        let heat_proximity = ((b.y - HEATER_Y) / (1.0 - HEATER_Y)).clamp(0.0, 1.0);
        0.9 * heat_proximity * dt
      } else {
        0.0
      };
      b.temp = (b.temp - cooling + heating).clamp(0.0, 1.0);

      // Advance position.
      b.x += b.vx * dt;
      b.y += b.vy * dt;

      // Bounce off lamp walls (cosine-restitution = 0.4 so it does not look
      // like a billiard ball).
      let lw = widths[i];
      if b.x < 0.5 - lw {
        b.x = 0.5 - lw;
        b.vx = b.vx.abs() * 0.4;
      } else if b.x > 0.5 + lw {
        b.x = 0.5 + lw;
        b.vx = -b.vx.abs() * 0.4;
      }
      if b.y > 0.96 {
        b.y = 0.96;
        b.vy = -b.vy.abs() * 0.3;
      } else if b.y < 0.04 {
        b.y = 0.04;
        b.vy = b.vy.abs() * 0.3;
      }
    }

    // Merge pairs that overlap by more than 30% of their combined radius.
    let mut i = 0;
    while i < self.blobs.len() {
      let mut merged = false;
      for j in (i + 1)..self.blobs.len() {
        let bi = self.blobs[i];
        let bj = self.blobs[j];
        let d = ((bi.x - bj.x).powi(2) + (bi.y - bj.y).powi(2)).sqrt();
        let r_total = (bi.radius() + bj.radius()) * 0.7;
        if d < r_total {
          let m = bi.mass + bj.mass;
          self.blobs[i] = Blob {
            x: (bi.x * bi.mass + bj.x * bj.mass) / m,
            y: (bi.y * bi.mass + bj.y * bj.mass) / m,
            vx: (bi.vx * bi.mass + bj.vx * bj.mass) / m,
            vy: (bi.vy * bi.mass + bj.vy * bj.mass) / m,
            mass: m,
            temp: (bi.temp * bi.mass + bj.temp * bj.mass) / m,
          };
          self.blobs.swap_remove(j);
          merged = true;
          break;
        }
      }
      if !merged {
        i += 1;
      }
    }

    // Splitting: large blobs split with low probability per frame.
    let n_split: Vec<(usize, f64)> = self
      .blobs
      .iter()
      .enumerate()
      .filter(|(_, b)| b.mass > SPLIT_THRESHOLD)
      .filter_map(|(i, _)| {
        let p = (0.15 * dt).min(0.05);
        if self.rng.gen::<f64>() < p {
          let angle: f64 = self.rng.gen_range(0.0..TAU);
          Some((i, angle))
        } else {
          None
        }
      })
      .collect();
    for (i, angle) in n_split {
      let parent = self.blobs[i];
      let daughter_mass = parent.mass * 0.4;
      let speed = 0.07;
      self.blobs[i].mass = parent.mass * 0.6;
      self.blobs[i].vx -= angle.cos() * speed * 0.3;
      self.blobs[i].vy -= angle.sin() * speed * 0.3;
      let separation = parent.radius() * 0.5;
      self.blobs.push(Blob {
        x: parent.x + angle.cos() * separation,
        y: parent.y + angle.sin() * separation,
        vx: parent.vx + angle.cos() * speed,
        vy: parent.vy + angle.sin() * speed,
        mass: daughter_mass,
        temp: parent.temp,
      });
    }

    // Cap blob count: drop the smallest.
    while self.blobs.len() > MAX_BLOBS {
      let min_idx = self
        .blobs
        .iter()
        .enumerate()
        .min_by(|a, b| a.1.mass.partial_cmp(&b.1.mass).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(i, _)| i)
        .unwrap_or(0);
      self.blobs.swap_remove(min_idx);
    }

    // Spawn a fresh small blob if we are running low.
    if self.blobs.len() < MIN_BLOBS && self.rng.gen::<f64>() < 1.5 * dt {
      self.spawn_fresh();
    }

    // Render: lamp glass + blob density field.
    let w = ctx.width;
    let h = ctx.height;
    let ax = w as f64 / (h as f64 * 2.0).max(1.0);
    let contrast = ctx.options.contrast;
    let mut grid = vec![0.0_f64; w * h];
    let dw = (w.saturating_sub(1)).max(1) as f64;
    let dh = (h.saturating_sub(1)).max(1) as f64;
    for row in 0..h {
      let v = row as f64 / dh;
      let lw = self.lamp_width(v);
      let base = row * w;
      for col in 0..w {
        let u = col as f64 / dw;
        let dx_wall = (u - 0.5).abs() - lw;
        let glass = (-((dx_wall * 35.0).powi(2))).exp() * 0.20;
        let mut value = glass;
        if dx_wall < 0.0 {
          // Interior glow + heater near bottom.
          value += 0.05 + 0.10 * (1.0 - v);
          if v > HEATER_Y {
            value += 0.20 * ((v - HEATER_Y) / (1.0 - HEATER_Y)).powi(2);
          }
          // Blob contributions: Gaussian weighted by temperature so cold
          // blobs are dimmer than hot ones.
          for b in &self.blobs {
            let dx = (u - b.x) * ax;
            let dy = v - b.y;
            let r = b.radius();
            let inv_r = 1.0 / r.max(0.001);
            let s = (dx * inv_r).powi(2) + (dy * inv_r).powi(2);
            let intensity = 0.30 + 0.65 * b.temp;
            value += intensity * (-s * 1.5).exp();
          }
        }
        grid[base + col] = clamp(value * contrast);
      }
    }
    render_field(ctx, &grid, TH, &STYLE, out);
  }
}
