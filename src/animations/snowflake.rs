//! Single hexagonal snowflake grown live via 6-fold-symmetric DLA, cycling
//! through morphologies that are visibly distinct.
//!
//! Algorithm: a walker spawns at a random angle in one 60° sector at the
//! crystal's outer edge, then random-walks in cell coordinates until it
//! touches an existing ice cell. On attachment its position is converted to
//! screen units (terminal cells are 2:1 so cell-x distance is halved),
//! rotated by k·60° for k = 0..5, converted back to cells, and frozen.
//! That C6 mirroring is what gives a real snowflake's six-fold symmetry.
//!
//! Three morphologies are cycled so different crystal types are clearly
//! visible within a minute or two of watching:
//!
//! * Stellar dendrite -- the classic six-armed star.
//! * Fern dendrite -- very fine, highly branched arms.
//! * Plate -- compact hexagonal disk with thick arms.
//!
//! Aspect: terminal cells are taller than wide (~2:1). All rotation /
//! distance math happens in *screen* units so the six-fold symmetry is
//! geometrically correct; cell units are used only for walker stepping and
//! grid indexing.

use std::f64::consts::{PI, TAU};

use rand::{Rng, SeedableRng};
use rand_pcg::Pcg32;

use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, render_field, FieldStyle};

const STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "ice" };
const TH: &[(f64, char)] = &[
  (0.04, ' '),
  (0.14, '.'),
  (0.26, ':'),
  (0.40, '-'),
  (0.54, '='),
  (0.68, '+'),
  (0.80, '*'),
  (0.92, '#'),
  (1.01, '@'),
];

const MAX_WALK: i64 = 6000;
const WALKERS_PER_FRAME: usize = 28;
const DISPLAY_SECONDS: f64 = 3.0;
const DISSOLVE_SECONDS: f64 = 2.0;
const DISSOLVE_RATE: f64 = 2.0;

/// Cells are ~2:1 (twice as tall as wide), so screen-space distances need
/// to be multiplied by this when going to cell-x and divided when going
/// from cell-x to screen-x.
const CELL_X_PER_SCREEN: f64 = 2.0;

#[derive(Clone, Copy)]
struct Morphology {
  name: &'static str,
  /// Walker spawn distance from centre as fraction of the crystal radius.
  /// Close (small) -> dense, far (large) -> sparse.
  spawn_fraction: f64,
  /// Probability of attaching when adjacent to existing ice. Low ->
  /// walker bounces back and explores further, filling gaps; high ->
  /// walker sticks at first contact, producing thin branches.
  stickiness: f64,
  /// Target crystal cell count before declaring done.
  max_cells: usize,
}

const MORPHOLOGIES: &[Morphology] = &[
  Morphology { name: "stellar dendrite", spawn_fraction: 0.95, stickiness: 0.85, max_cells: 1800 },
  Morphology { name: "fern dendrite", spawn_fraction: 0.97, stickiness: 0.99, max_cells: 1200 },
  Morphology { name: "plate", spawn_fraction: 0.50, stickiness: 0.30, max_cells: 2800 },
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
  Growing,
  Holding,
  Dissolving,
}

pub struct Snowflake {
  ice: Vec<bool>,
  age: Vec<f64>,
  /// Brightness buffer for the dissolve phase; decays per frame so the
  /// previous crystal fades cleanly before the next starts.
  glow: Vec<f64>,
  w: usize,
  h: usize,
  last: f64,
  hold_until: f64,
  dissolve_until: f64,
  rng: Pcg32,
  phase: Phase,
  morph_idx: usize,
  cell_count: usize,
}

impl Default for Snowflake {
  fn default() -> Self {
    Self {
      ice: Vec::new(),
      age: Vec::new(),
      glow: Vec::new(),
      w: 0,
      h: 0,
      last: 0.0,
      hold_until: 0.0,
      dissolve_until: 0.0,
      rng: Pcg32::seed_from_u64(0x5F0C_1ADE_BABE),
      phase: Phase::Growing,
      morph_idx: 0,
      cell_count: 0,
    }
  }
}

impl Snowflake {
  fn reseed(&mut self, w: usize, h: usize) {
    self.ice = vec![false; w * h];
    self.age = vec![0.0; w * h];
    self.glow = vec![0.0; w * h];
    self.w = w;
    self.h = h;
    self.phase = Phase::Growing;
    self.cell_count = 0;
    let cx = w / 2;
    let cy = h / 2;
    let idx = cy * w + cx;
    self.ice[idx] = true;
    self.glow[idx] = 1.0;
    self.cell_count = 1;
  }

  fn morphology(&self) -> Morphology {
    MORPHOLOGIES[self.morph_idx % MORPHOLOGIES.len()]
  }

  /// (radius in cell-x, radius in cell-y) so the result is a circle in
  /// screen units. Picks whichever screen dimension is smaller as the
  /// constraint, taking 40% of it so the crystal fits with margin.
  fn crystal_radius_cells(&self) -> (f64, f64) {
    // 1 cell-y = 1 screen unit (cells are 2:1).
    // 1 screen unit horizontally = CELL_X_PER_SCREEN cell-x.
    let max_screen_units = (self.w as f64 * 0.42 / CELL_X_PER_SCREEN).min(self.h as f64 * 0.42);
    let r_cell_x = max_screen_units * CELL_X_PER_SCREEN;
    let r_cell_y = max_screen_units;
    (r_cell_x, r_cell_y)
  }

  /// Spawn one walker, walk it, attempt to attach. Returns true if it stuck.
  fn launch_walker(&mut self, t: f64) -> bool {
    let m = self.morphology();
    let (rx, ry) = self.crystal_radius_cells();
    let cx = self.w as f64 * 0.5;
    let cy = self.h as f64 * 0.5;

    // Spawn at random angle in the first 60° sector.
    let theta: f64 = self.rng.gen_range(0.0..PI / 3.0);
    let mut x = cx + rx * m.spawn_fraction * theta.cos();
    let mut y = cy + ry * m.spawn_fraction * theta.sin();

    for _ in 0..MAX_WALK {
      // 8-neighbour random step in cell coords.
      let dx_step: i64 = self.rng.gen_range(-1..=1);
      let dy_step: i64 = self.rng.gen_range(-1..=1);
      if dx_step == 0 && dy_step == 0 {
        continue;
      }
      x += dx_step as f64;
      y += dy_step as f64;
      let xi = x.round() as i64;
      let yi = y.round() as i64;
      if xi < 1 || xi >= self.w as i64 - 1 || yi < 1 || yi >= self.h as i64 - 1 {
        return false;
      }
      if !self.touches_ice(xi, yi) {
        continue;
      }
      // Stickiness gate.
      if self.rng.gen::<f64>() > m.stickiness {
        // Bounce back one step and keep walking.
        x -= dx_step as f64;
        y -= dy_step as f64;
        continue;
      }
      // Attach. Mirror to all 6 sectors via screen-space rotation so the
      // C6 symmetry is geometrically correct.
      self.freeze_six(xi, yi, t);
      return true;
    }
    false
  }

  fn touches_ice(&self, xi: i64, yi: i64) -> bool {
    for dy in -1..=1_i64 {
      for dx in -1..=1_i64 {
        if dx == 0 && dy == 0 {
          continue;
        }
        let nx = xi + dx;
        let ny = yi + dy;
        if nx < 0 || nx >= self.w as i64 || ny < 0 || ny >= self.h as i64 {
          continue;
        }
        if self.ice[ny as usize * self.w + nx as usize] {
          return true;
        }
      }
    }
    false
  }

  fn freeze_six(&mut self, cell_x: i64, cell_y: i64, t: f64) {
    let cx_cell = self.w as f64 * 0.5;
    let cy_cell = self.h as f64 * 0.5;
    // Offset in screen units (cell-x scaled down because cells are 2:1).
    let dx_screen = (cell_x as f64 - cx_cell) / CELL_X_PER_SCREEN;
    let dy_screen = cell_y as f64 - cy_cell;
    for k in 0..6 {
      let phi = TAU * k as f64 / 6.0;
      let (cs, sn) = (phi.cos(), phi.sin());
      let rx_screen = dx_screen * cs - dy_screen * sn;
      let ry_screen = dx_screen * sn + dy_screen * cs;
      // Convert back to cell coords.
      let sx = (cx_cell + rx_screen * CELL_X_PER_SCREEN).round() as i64;
      let sy = (cy_cell + ry_screen).round() as i64;
      if sx < 0 || sx >= self.w as i64 || sy < 0 || sy >= self.h as i64 {
        continue;
      }
      let idx = sy as usize * self.w + sx as usize;
      if !self.ice[idx] {
        self.ice[idx] = true;
        self.age[idx] = t;
        self.glow[idx] = 1.0;
        self.cell_count += 1;
      }
    }
  }
}

impl Animation for Snowflake {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let w = ctx.width;
    let h = ctx.height;
    if self.ice.is_empty() || ctx.elapsed < self.last {
      self.reseed(w, h);
    } else if w != self.w || h != self.h {
      // Crystal would be at the wrong scale -- restart cleanly.
      self.reseed(w, h);
    }
    let dt = (ctx.elapsed - self.last).clamp(0.0, 0.1);
    self.last = ctx.elapsed;
    let m = self.morphology();

    match self.phase {
      Phase::Growing => {
        for _ in 0..WALKERS_PER_FRAME {
          self.launch_walker(ctx.elapsed);
        }
        if self.cell_count >= m.max_cells {
          self.phase = Phase::Holding;
          self.hold_until = ctx.elapsed + DISPLAY_SECONDS;
        }
      }
      Phase::Holding => {
        if ctx.elapsed >= self.hold_until {
          self.phase = Phase::Dissolving;
          self.dissolve_until = ctx.elapsed + DISSOLVE_SECONDS;
        }
      }
      Phase::Dissolving => {
        let decay = (-dt * DISSOLVE_RATE).exp();
        for g in self.glow.iter_mut() {
          *g *= decay;
        }
        if ctx.elapsed >= self.dissolve_until {
          self.morph_idx = self.morph_idx.wrapping_add(1);
          self.reseed(w, h);
        }
      }
    }

    let contrast = ctx.options.contrast;
    let mut grid = vec![0.0_f64; w * h];
    let cx_f = w as f64 * 0.5;
    let cy_f = h as f64 * 0.5;
    let (max_rx, _) = self.crystal_radius_cells();
    for row in 0..h {
      let base = row * w;
      for col in 0..w {
        let idx = base + col;
        let bright = match self.phase {
          Phase::Growing | Phase::Holding => {
            if !self.ice[idx] {
              0.0
            } else {
              let recency = (ctx.elapsed - self.age[idx]).max(0.0);
              // Freshly-grown cells flash brighter so the growth front is
              // visible.
              let freshness = (1.0 - recency / 1.5).clamp(0.0, 1.0);
              // Mild radial darkening so the core reads as solid and the
              // tips as the brightest point.
              let dx = (col as f64 - cx_f) / CELL_X_PER_SCREEN;
              let dy = row as f64 - cy_f;
              let r = (dx * dx + dy * dy).sqrt() / (max_rx / CELL_X_PER_SCREEN).max(1.0);
              let radial = (1.0 - 0.25 * r).clamp(0.45, 1.0);
              (0.70 + 0.30 * freshness) * radial
            }
          }
          Phase::Dissolving => self.glow[idx],
        };
        grid[idx] = clamp(bright * contrast);
      }
    }
    render_field(ctx, &grid, TH, &STYLE, out);
  }

  fn status(&self) -> Option<String> {
    let phase = match self.phase {
      Phase::Growing => "growing",
      Phase::Holding => "holding",
      Phase::Dissolving => "dissolving",
    };
    Some(format!("{} | {} | {} cells", self.morphology().name, phase, self.cell_count))
  }
}
