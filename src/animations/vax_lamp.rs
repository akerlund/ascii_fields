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

const MAX_BLOBS: usize = 12;
const MIN_BLOBS: usize = 5;
// Blob can split once it has roughly twice the starting mass. Earlier
// threshold means the population dynamics turn over fast enough to be
// visible -- waiting for mass 1.4 meant blobs rarely got there.
const SPLIT_THRESHOLD: f64 = 0.9;
const HEATER_Y: f64 = 0.82; // y > this is in the heater zone (y=1.0 is bottom)

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
    // Widened from the previous 0.13 base + 0.08 bell so blobs have room
    // to move sideways instead of being trapped in a narrow column.
    let bell = (1.0 - (y - 0.5).powi(2) * 4.0).clamp(0.05, 1.0);
    0.20 + 0.14 * bell
  }

  fn spawn_fresh(&mut self) {
    // Wide x range and meaningful initial vx so fresh blobs don't all
    // pile up in the centre and stay there.
    let x = self.rng.gen_range(0.30..0.70);
    let y = self.rng.gen_range(0.80..0.95);
    self.blobs.push(Blob {
      x,
      y,
      vx: self.rng.gen_range(-0.08..0.08),
      vy: 0.0,
      mass: self.rng.gen_range(0.35..0.70),
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

    // Physics integration. Tuned so blobs actually reach the top, move
    // sideways visibly, and collide before merging:
    //
    //   * Stronger buoyancy (0.45 -> 0.80) so hot blobs accelerate enough to
    //     traverse the lamp before cooling.
    //   * Slower cooling (0.25 -> 0.12) so they retain heat past the
    //     midpoint and finish the trip up.
    //   * Stronger heater (0.90 -> 1.60) so cold blobs reheat fast at the
    //     bottom and start the cycle again.
    //   * Much lower drag (0.60 -> 0.18) so horizontal Brownian motion
    //     accumulates into visible side travel instead of being damped.
    //   * Larger jiggle range so the side motion is meaningful, not noise.
    for (i, b) in self.blobs.iter_mut().enumerate() {
      // Buoyancy is now PROPORTIONAL TO MASS: bigger blobs accelerate
      // harder, smaller blobs rise lazily. Real fluid physics would say
      // acceleration is mass-invariant, but for the visual the user
      // wants (small lumbering, big sweeping), scaling by mass gives
      // the right feel.
      let buoyancy = -(b.temp - 0.45) * 0.55 * b.mass;
      b.vy += buoyancy * dt;

      let jiggle: f64 = self.rng.gen_range(-0.30..0.30);
      b.vx += jiggle * dt;

      b.vx *= 1.0 - 0.18 * dt;
      b.vy *= 1.0 - 0.18 * dt;

      let cooling = 0.12 * dt;
      let heating = if b.y > HEATER_Y {
        let heat_proximity = ((b.y - HEATER_Y) / (1.0 - HEATER_Y)).clamp(0.0, 1.0);
        1.60 * heat_proximity * dt
      } else {
        0.0
      };
      b.temp = (b.temp - cooling + heating).clamp(0.0, 1.0);

      b.x += b.vx * dt;
      b.y += b.vy * dt;

      // Bounce off lamp walls (restitution 0.4).
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

    // Heat exchange between nearby blobs that haven't merged. When a cold
    // blob falls past a warm one, they trade temperature -- the warm one
    // cools, the cold one heats up, just like real wax blobs in convection.
    // Newton's law of cooling between each pair, weighted by proximity.
    let n = self.blobs.len();
    let mut delta_temp = vec![0.0_f64; n];
    for i in 0..n {
      for j in (i + 1)..n {
        let bi = self.blobs[i];
        let bj = self.blobs[j];
        let d = ((bi.x - bj.x).powi(2) + (bi.y - bj.y).powi(2)).sqrt();
        let interaction_range = (bi.radius() + bj.radius()) * 1.5;
        if d < interaction_range {
          // Linear falloff in proximity weight.
          let proximity = 1.0 - (d / interaction_range);
          let exchange_rate = 0.6 * dt * proximity;
          let diff = bi.temp - bj.temp;
          delta_temp[i] -= exchange_rate * diff;
          delta_temp[j] += exchange_rate * diff;
        }
      }
    }
    for (i, dt_val) in delta_temp.iter().enumerate() {
      self.blobs[i].temp = (self.blobs[i].temp + dt_val).clamp(0.0, 1.0);
    }

    // Merge pairs only when they overlap significantly (centres within
    // 50% of combined radii). Earlier they merged the moment their
    // edges touched, so the viewer never saw them deform around each
    // other -- they just instantly combined.
    let mut i = 0;
    while i < self.blobs.len() {
      let mut merged = false;
      for j in (i + 1)..self.blobs.len() {
        let bi = self.blobs[i];
        let bj = self.blobs[j];
        let d = ((bi.x - bj.x).powi(2) + (bi.y - bj.y).powi(2)).sqrt();
        let r_total = (bi.radius() + bj.radius()) * 0.5;
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
          // Blob contributions: Gaussian weighted by temperature, with
          // strong velocity-driven stretching so fast-moving blobs
          // deform into clear teardrop shapes. The "along" axis goes
          // with velocity (elongated), the "across" axis is squashed.
          // Also offsets the bright centre slightly opposite the
          // motion direction so each blob has a visible "trailing tail"
          // -- the bulk-of-mass sits behind the leading edge, like a
          // real rising wax blob.
          for b in &self.blobs {
            let dx = (u - b.x) * ax;
            let dy = v - b.y;
            let r = b.radius();
            // Speed -> stretch factor. Much more dramatic than before
            // (max stretch 3.0 instead of 2.2, scale coefficient 7.0
            // instead of 4.5). Resting blobs are still round, but any
            // meaningful motion now produces a visibly elongated shape.
            let speed = (b.vx * b.vx + b.vy * b.vy).sqrt();
            let stretch = (1.0 + 7.0 * speed).min(3.0);
            let inv_along = 1.0 / (r * stretch).max(0.001);
            let inv_across = 1.0 / (r / stretch.sqrt()).max(0.001);
            let speed_safe = speed.max(1e-6);
            let along_x = b.vx / speed_safe;
            let along_y = b.vy / speed_safe;
            // Shift the projection so the bright centre is offset behind
            // the motion -- the blob looks like a teardrop with a tail
            // pointing back where it came from.
            let trail_offset = r * 0.5 * (stretch - 1.0);
            let shifted_dx = dx + along_x * trail_offset;
            let shifted_dy = dy + along_y * trail_offset;
            let along = shifted_dx * along_x + shifted_dy * along_y;
            let across = shifted_dx * (-along_y) + shifted_dy * along_x;
            let s = (along * inv_along).powi(2) + (across * inv_across).powi(2);
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
