//! Falling snow: multiple hex-lattice snowflakes drifting down and rotating,
//! each one procedurally generated with a unique branching pattern.
//!
//! A real snowflake is hexagonal because water ice crystallizes on a hex
//! lattice. We work in axial hex coordinates (q, r); the six unit directions
//! are
//!
//!     d0 = ( 1,  0)   d3 = (-1,  0)
//!     d1 = ( 0,  1)   d4 = ( 0, -1)
//!     d2 = (-1,  1)   d5 = ( 1, -1)
//!
//! A snowflake shape is built by:
//!
//!   1. Generating cells in ONE 60° wedge with random branching rules
//!      (main arm length, side-branch probability, secondary branches).
//!   2. Mirroring the wedge to all six sectors via the axial-rotation
//!      formula (q, r) -> (-r, q+r) applied 0..6 times.
//!
//! The result is a hexagonally symmetric crystal with arms along the six
//! hex directions and side branches at 60°, exactly the structure real
//! snowflakes have.
//!
//! Rendering: for each falling flake, we transform every hex cell by
//! (rotate by the flake's current angle) -> (scale by flake size) ->
//! (translate to flake's screen position), and plot the result at one
//! brightness intensity. Rendering many flakes at different scales gives
//! a parallax depth illusion -- big ones look near, small ones look far.

use std::collections::HashSet;
use std::f64::consts::TAU;

use rand::{Rng, SeedableRng};
use rand_pcg::Pcg32;

use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, render_field, FieldStyle};

const STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "ice" };
const TH: &[(f64, char)] = &[
  (0.04, ' '),
  (0.16, '.'),
  (0.30, ':'),
  (0.46, '-'),
  (0.60, '+'),
  (0.74, '*'),
  (0.85, '#'),
  (0.95, '@'),
  (1.01, '@'),
];

const SQRT3: f64 = 1.732_050_807_568_877_2;
const MAX_FLAKES: usize = 14;

// ---------------------------------------------------------------------------
// Hex shape generation
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct SnowflakeShape {
  /// All cells in axial (q, r) coordinates, including 6-fold mirrored copies.
  cells: Vec<(i32, i32)>,
  /// Bounding hex-axial radius (max distance from centre of any cell).
  bound_radius: f64,
}

/// Rotate an axial hex coordinate by 60° clockwise: (q, r) -> (-r, q+r).
#[inline]
fn rotate_hex(q: i32, r: i32, steps: i32) -> (i32, i32) {
  let s = steps.rem_euclid(6);
  let (mut q, mut r) = (q, r);
  for _ in 0..s {
    let (nq, nr) = (-r, q + r);
    q = nq;
    r = nr;
  }
  (q, r)
}

/// Hex axial -> Cartesian (in hex-radius units, so a single hex side = 1).
#[inline]
fn axial_to_xy(q: i32, r: i32) -> (f64, f64) {
  let qf = q as f64;
  let rf = r as f64;
  (SQRT3 * (qf + rf * 0.5), 1.5 * rf)
}

impl SnowflakeShape {
  fn generate(rng: &mut Pcg32) -> Self {
    // One 60° wedge: contains the cells between hex directions d0 and d1.
    // Cells in the wedge have q >= 0 and r >= 0.
    let mut wedge: HashSet<(i32, i32)> = HashSet::new();
    wedge.insert((0, 0));

    // Main-arm parameters.
    let main_length = rng.gen_range(4..=11);
    let main_density: f64 = rng.gen_range(0.85..=1.0);

    // Side branches off the main arm. They go in the (0, 1) direction
    // (60° from the main arm), which stays inside the wedge.
    let branch_prob: f64 = rng.gen_range(0.20..0.75);
    let branch_max_len = rng.gen_range(1..=4);
    let secondary_prob: f64 = rng.gen_range(0.0..0.40);

    // Optional "plate fill": for plate-type crystals, fill some inner cells.
    let plate_fill_inner: i32 = if rng.gen::<f64>() < 0.30 { rng.gen_range(0..=2) } else { 0 };

    // Main arm along d0.
    for i in 1..=main_length {
      if rng.gen::<f64>() < main_density {
        wedge.insert((i, 0));
      }
    }

    // Side branches.
    for i in 2..main_length {
      if !wedge.contains(&(i, 0)) {
        continue;
      }
      if rng.gen::<f64>() < branch_prob {
        let len = rng.gen_range(1..=branch_max_len);
        for j in 1..=len {
          let cell = (i, j);
          wedge.insert(cell);
          // Possibly a sub-branch back outward (in d0 direction).
          if j > 1 && rng.gen::<f64>() < secondary_prob {
            let sub_len = rng.gen_range(1..=2);
            for k in 1..=sub_len {
              wedge.insert((cell.0 + k, cell.1));
            }
          }
        }
      }
    }

    // Plate fill: add the innermost rings of the wedge.
    for q in 0..=plate_fill_inner {
      for r in 0..=plate_fill_inner {
        if q + r <= plate_fill_inner {
          wedge.insert((q, r));
        }
      }
    }

    // Mirror wedge into all 6 sectors.
    let mut all: HashSet<(i32, i32)> = HashSet::new();
    for &(q, r) in &wedge {
      for k in 0..6 {
        all.insert(rotate_hex(q, r, k));
      }
    }

    let cells: Vec<(i32, i32)> = all.into_iter().collect();
    let bound_radius = cells
      .iter()
      .map(|&(q, r)| {
        let (x, y) = axial_to_xy(q, r);
        (x * x + y * y).sqrt()
      })
      .fold(0.0_f64, f64::max);

    Self { cells, bound_radius }
  }
}

// ---------------------------------------------------------------------------
// Falling flakes
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct Flake {
  shape: SnowflakeShape,
  cx: f64,
  cy: f64,
  scale: f64,
  rotation: f64,
  omega: f64,
  vy: f64,
  wobble_amp: f64,
  wobble_freq: f64,
  wobble_phase: f64,
  brightness: f64,
}

pub struct Snowflake {
  flakes: Vec<Flake>,
  next_spawn: f64,
  last: f64,
  w: usize,
  h: usize,
  rng: Pcg32,
}

impl Default for Snowflake {
  fn default() -> Self {
    Self {
      flakes: Vec::new(),
      next_spawn: 0.0,
      last: 0.0,
      w: 0,
      h: 0,
      rng: Pcg32::seed_from_u64(0x5F0C_1ADE_BABE),
    }
  }
}

impl Snowflake {
  fn spawn_flake(&mut self) {
    let shape = SnowflakeShape::generate(&mut self.rng);
    // Bigger flakes fall slower and look brighter (closer to viewer).
    // Pick a random "depth" parameter that controls scale + speed + bright.
    let depth: f64 = self.rng.gen_range(0.3..1.0);
    let scale = 0.6 + depth * 2.2;
    let vy = 2.0 + (1.0 - depth) * 5.0; // far flakes (small depth) fall faster
    let brightness = 0.40 + depth * 0.55;
    let cx: f64 = self.rng.gen_range(0.0..self.w as f64);
    let cy = -shape.bound_radius * scale * 1.5;
    self.flakes.push(Flake {
      shape,
      cx,
      cy,
      scale,
      rotation: self.rng.gen_range(0.0..TAU),
      omega: self.rng.gen_range(-0.35..0.35),
      vy,
      wobble_amp: self.rng.gen_range(0.0..1.6),
      wobble_freq: self.rng.gen_range(0.4..0.9),
      wobble_phase: self.rng.gen_range(0.0..TAU),
      brightness,
    });
  }
}

impl Animation for Snowflake {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let w = ctx.width;
    let h = ctx.height;
    if w != self.w || h != self.h || ctx.elapsed < self.last {
      // First frame, resize, or rewind: clear and reseed.
      self.flakes.clear();
      self.next_spawn = ctx.elapsed;
      self.w = w;
      self.h = h;
      // Pre-populate so the screen is not empty for the first ~5 seconds.
      // Each fresh flake is placed at a random y, not above the screen.
      for _ in 0..MAX_FLAKES / 2 {
        self.spawn_flake();
        if let Some(f) = self.flakes.last_mut() {
          f.cy = self.rng.gen_range(0.0..h as f64);
        }
      }
    }
    let dt = (ctx.elapsed - self.last).clamp(0.0, 0.1);
    self.last = ctx.elapsed;

    // Spawn new flakes at a steady rate.
    while ctx.elapsed >= self.next_spawn {
      if self.flakes.len() < MAX_FLAKES {
        self.spawn_flake();
      }
      // Inter-arrival ~0.4..1.0s.
      let next: f64 = self.rng.gen_range(0.40..1.00);
      self.next_spawn += next;
    }

    // Advance each flake.
    for f in &mut self.flakes {
      f.cy += f.vy * dt;
      f.rotation += f.omega * dt;
      // Horizontal wobble around the spawn x, applied as a phase offset.
      let wobble = f.wobble_amp * (ctx.elapsed * f.wobble_freq * TAU + f.wobble_phase).sin();
      f.cx += wobble * dt * 0.6;
    }

    // Cull flakes that have fallen off the bottom or drifted off the sides.
    self.flakes.retain(|f| {
      let margin = f.shape.bound_radius * f.scale * 2.0;
      f.cy < h as f64 + margin && f.cx > -margin && f.cx < w as f64 + margin
    });

    // Render.
    let contrast = ctx.options.contrast;
    let mut grid = vec![0.0_f64; w * h];
    let cell_aspect = 2.0_f64; // terminal cells are ~2:1 (tall to wide)
    for f in &self.flakes {
      let (cs, sn) = (f.rotation.cos(), f.rotation.sin());
      for &(q, r) in &f.shape.cells {
        let (hx, hy) = axial_to_xy(q, r);
        let scaled_x = hx * f.scale;
        let scaled_y = hy * f.scale;
        // Rotation in screen space (so the snowflake stays geometrically
        // hexagonal regardless of terminal cell aspect).
        let rsx = scaled_x * cs - scaled_y * sn;
        let rsy = scaled_x * sn + scaled_y * cs;
        // Screen-space to cell-space: cell-x is half the size of cell-y, so
        // multiply screen-x by 2 to get cell-x.
        let col = (f.cx + rsx * cell_aspect).round() as i64;
        let row = (f.cy + rsy).round() as i64;
        if col < 0 || col >= w as i64 || row < 0 || row >= h as i64 {
          continue;
        }
        let idx = row as usize * w + col as usize;
        if grid[idx] < f.brightness {
          grid[idx] = f.brightness;
        }
      }
    }
    for v in grid.iter_mut() {
      *v = clamp(*v * contrast);
    }
    render_field(ctx, &grid, TH, &STYLE, out);
  }

  fn status(&self) -> Option<String> {
    Some(format!("{} flakes", self.flakes.len()))
  }
}
