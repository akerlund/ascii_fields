//! Double pendulum ensemble -- the classical visceral demonstration of
//! chaos. We simulate ~12 double pendulums whose initial angles differ
//! by a tiny amount (~0.0005 rad). For the first second or so they swing
//! in near-unison, then exponential divergence kicks in and within ~5
//! seconds each pendulum is drawing a completely different trajectory.
//!
//! Each pendulum tip leaves a fading colour trail; the brightness of a
//! trail point biases it into a different palette band per pendulum, so
//! the divergence is visible as the previously-overlapping rainbows
//! peeling apart.
//!
//! Integration: simple semi-implicit Euler. The double pendulum equations
//! are stiff over long horizons but for the screen-time of a visualization
//! Euler with small dt looks fine.

use std::f64::consts::TAU;

use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, render_field, FieldStyle};

const STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "spectrum" };
const TH: &[(f64, char)] = &[
  (0.05, ' '),
  (0.14, '.'),
  (0.24, ':'),
  (0.36, '-'),
  (0.48, '='),
  (0.60, '+'),
  (0.74, '*'),
  (0.88, '#'),
  (1.01, '@'),
];

const N_PENDULUMS: usize = 12;
const SUB_STEPS: usize = 8;
const G: f64 = 9.81;
const L1: f64 = 1.0;
const L2: f64 = 1.0;
const M1: f64 = 1.0;
const M2: f64 = 1.0;
/// Per-second decay applied to the trail accumulator. Slower than the
/// initial value so trails persist long enough to be bright without the
/// user cranking --contrast.
const FADE_RATE: f64 = 0.3;

#[derive(Clone, Copy)]
struct DoublePendulum {
  theta1: f64,
  theta2: f64,
  omega1: f64,
  omega2: f64,
}

impl DoublePendulum {
  /// One forward Euler integration step on the standard double-pendulum
  /// equations.
  fn step(&mut self, dt: f64) {
    let s1 = self.theta1.sin();
    let s2 = self.theta2.sin();
    let s12 = (self.theta1 - self.theta2).sin();
    let c12 = (self.theta1 - self.theta2).cos();

    let denom1 = L1 * (2.0 * M1 + M2 - M2 * (2.0 * (self.theta1 - self.theta2)).cos());
    let num1 = -G * (2.0 * M1 + M2) * s1
      - M2 * G * (self.theta1 - 2.0 * self.theta2).sin()
      - 2.0 * s12 * M2 * (self.omega2.powi(2) * L2 + self.omega1.powi(2) * L1 * c12);
    let alpha1 = num1 / denom1;

    let denom2 = L2 * (2.0 * M1 + M2 - M2 * (2.0 * (self.theta1 - self.theta2)).cos());
    let num2 = 2.0
      * s12
      * (self.omega1.powi(2) * L1 * (M1 + M2)
        + G * (M1 + M2) * self.theta1.cos()
        + self.omega2.powi(2) * L2 * M2 * c12);
    let alpha2 = num2 / denom2;
    let _ = s2;

    self.omega1 += alpha1 * dt;
    self.omega2 += alpha2 * dt;
    self.theta1 += self.omega1 * dt;
    self.theta2 += self.omega2 * dt;
  }
}

pub struct Chaos {
  pendulums: Vec<DoublePendulum>,
  accumulator: Vec<f64>,
  w: usize,
  h: usize,
  last: f64,
}

impl Default for Chaos {
  fn default() -> Self {
    let mut pendulums = Vec::with_capacity(N_PENDULUMS);
    let theta0 = 2.4; // ~140 degrees from rest, well into chaotic regime
    for i in 0..N_PENDULUMS {
      // Initial angle differs by 0.0005 rad per pendulum -- tiny enough
      // that the first second of motion looks coordinated, but enough
      // for the exponential divergence to spread them apart in seconds.
      let delta = 0.0005 * i as f64;
      pendulums.push(DoublePendulum {
        theta1: theta0 + delta,
        theta2: theta0 + delta,
        omega1: 0.0,
        omega2: 0.0,
      });
    }
    Self { pendulums, accumulator: Vec::new(), w: 0, h: 0, last: 0.0 }
  }
}

impl Animation for Chaos {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let w = ctx.width;
    let h = ctx.height;
    if w != self.w || h != self.h || ctx.elapsed < self.last {
      self.accumulator = vec![0.0_f64; w * h];
      self.w = w;
      self.h = h;
      // Re-init pendulums on a reset so the divergence reseeds visibly.
      *self = Self { accumulator: vec![0.0_f64; w * h], w, h, last: ctx.elapsed, ..Default::default() };
    }
    let dt = (ctx.elapsed - self.last).clamp(0.0, 0.1);
    self.last = ctx.elapsed;
    let decay = (-dt * FADE_RATE).exp();
    for v in self.accumulator.iter_mut() {
      *v *= decay;
    }

    // Sub-stepped integration to keep Euler stable.
    let step_dt = dt / SUB_STEPS as f64;
    for p in &mut self.pendulums {
      for _ in 0..SUB_STEPS {
        p.step(step_dt);
      }
    }

    let ax = w as f64 / (h as f64 * 2.0).max(1.0);
    let dw = (w.saturating_sub(1)).max(1) as f64;
    let dh = (h.saturating_sub(1)).max(1) as f64;
    let cx = 0.5;
    // Pivot in the upper third so the pendulum's resting position
    // (hanging straight down) is near the bottom of the screen, and
    // the inverted swing reaches close to the top.
    let cy = 0.30;
    let arm_scale = 0.28;

    for (i, p) in self.pendulums.iter().enumerate() {
      // Position of the second-arm tip in world coords. tip_y is the
      // mathematical y-coordinate (positive = up, negative = down). To
      // map to screen (where positive y is DOWN), subtract.
      let tip_x = L1 * p.theta1.sin() + L2 * p.theta2.sin();
      let tip_y = -L1 * p.theta1.cos() - L2 * p.theta2.cos();
      let su = cx + (tip_x * arm_scale) / ax;
      let sv = cy - tip_y * arm_scale;
      if !(0.0..1.0).contains(&su) || !(0.0..1.0).contains(&sv) {
        continue;
      }
      // Per-pendulum brightness offset so the spectrum palette paints
      // each pendulum's trail in a different colour band -- divergence
      // becomes visible as the previously-overlapping rainbows peel
      // apart.
      let band_centre = 0.35 + 0.55 * (i as f64 / (N_PENDULUMS - 1) as f64);
      let cxi = (su * dw) as i64;
      let cyi = (sv * dh) as i64;
      for dy in -1..=1_i64 {
        let yy = cyi + dy;
        if yy < 0 || yy >= h as i64 {
          continue;
        }
        for dx in -1..=1_i64 {
          let xx = cxi + dx;
          if xx < 0 || xx >= w as i64 {
            continue;
          }
          let dist2 = (dx * dx + dy * dy) as f64;
          let g = (-dist2 / 0.65).exp();
          let idx = yy as usize * w + xx as usize;
          // Each visit sets the cell to at least the pendulum's band
          // centre weighted by the Gaussian footprint. With slow fade,
          // trails are bright enough at default contrast.
          let contribution = band_centre * g;
          if self.accumulator[idx] < contribution {
            self.accumulator[idx] = contribution;
          }
        }
      }
    }

    let contrast = ctx.options.contrast;
    let mut grid = vec![0.0_f64; w * h];
    for (g, &v) in grid.iter_mut().zip(self.accumulator.iter()) {
      *g = clamp(v * contrast);
    }
    render_field(ctx, &grid, TH, &STYLE, out);
    // TAU is unused locally; expose it as a no-op reference so the import
    // does not get pruned -- needed if we later add angle wraparound.
    let _ = TAU;
  }
}
