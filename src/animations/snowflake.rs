//! Single hexagonal snowflake grown live via 6-fold-symmetric DLA.
//!
//! Algorithm: a walker spawns at a random angle within one 60° sector at the
//! current spawn radius, then random-walks until it touches an existing ice
//! cell. On attachment its position (relative to the centre) is mirrored to
//! all six rotational copies — that's the explicit C6 symmetry every real
//! snowflake exhibits because of the hexagonal packing of water ice.
//!
//! Different crystal morphologies emerge from different parameter sets:
//!
//! * Plate: close spawn radius, low stickiness; walkers fill in gaps, giving
//!   solid sectors.
//! * Stellar: medium spawn radius, high stickiness; clean six-arm star.
//! * Fern: far spawn radius, high stickiness; thin highly-branched arms.
//!
//! We cycle through three parameter sets back-to-back so a viewer sees the
//! three classical crystal types within a couple of minutes.
//!
//! Aspect: terminal cells are about 2:1 (wider than tall). The walker math
//! works in world coordinates with `ax = w / (h * 2)` correction so the
//! crystal looks geometrically symmetric on screen instead of horizontally
//! stretched.

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

/// One walker's max steps before we give up and pretend it never attached.
const MAX_WALK: i64 = 4000;
/// How many walkers per frame -- determines crystal growth rate.
const WALKERS_PER_FRAME: usize = 18;
/// Seconds the crystal stays visible after it has stopped growing, before
/// dissolving and starting the next morphology.
const DISPLAY_SECONDS: f64 = 4.0;
/// Per-second decay applied during the dissolve phase so the previous
/// crystal fades away cleanly before the new one starts.
const DISSOLVE_RATE: f64 = 1.6;

#[derive(Clone, Copy)]
struct Morphology {
  /// Walker spawn distance from centre, as a fraction of the available
  /// crystal radius. Closer = denser, farther = sparser dendrites.
  spawn_fraction: f64,
  /// Probability of attaching when adjacent to ice (vs. bouncing). Higher
  /// = thinner more-branched dendrites; lower = thicker plates.
  stickiness: f64,
  /// Maximum cells in the crystal before we declare it done. Sets the
  /// crystal's overall size.
  max_cells: usize,
}

const MORPHOLOGIES: &[Morphology] = &[
  // Stellar dendrite -- the classic snowflake.
  Morphology { spawn_fraction: 0.95, stickiness: 0.90, max_cells: 2200 },
  // Fern dendrite -- very fine branching.
  Morphology { spawn_fraction: 0.98, stickiness: 0.98, max_cells: 1600 },
  // Plate -- compact six-petalled crystal.
  Morphology { spawn_fraction: 0.65, stickiness: 0.45, max_cells: 3000 },
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
  /// Brightness field that lingers during the Dissolving phase, kept
  /// separately from `age` so we can decay it without forgetting which
  /// cells were once ice.
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
    // Seed the centre cell as ice so walkers have something to attach to.
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

  /// Crystal radius in world (aspect-corrected) units. Walkers spawn at a
  /// fraction of this, and we use it to gate when the crystal is "full".
  fn crystal_radius(&self, ax: f64) -> f64 {
    0.42 * (self.w as f64 * 0.5 / ax).min(self.h as f64 * 0.5)
  }

  /// One walker. Returns true if it attached.
  fn launch_walker(&mut self, t: f64, ax: f64) -> bool {
    let m = self.morphology();
    let cx = self.w as f64 * 0.5;
    let cy = self.h as f64 * 0.5;
    let radius = self.crystal_radius(ax) * m.spawn_fraction;

    // Spawn at a random angle in the first 60° sector.
    let sector_a: f64 = self.rng.gen_range(0.0..PI / 3.0);
    let dx_world = radius * sector_a.cos();
    let dy_world = radius * sector_a.sin();
    let mut x = cx + dx_world / ax;
    let mut y = cy + dy_world;

    for _ in 0..MAX_WALK {
      // Random walk step, isotropic in world coords.
      let step_angle: f64 = self.rng.gen_range(0.0..TAU);
      x += step_angle.cos() / ax;
      y += step_angle.sin();
      let xi = x.round() as i64;
      let yi = y.round() as i64;
      if xi < 1 || xi >= self.w as i64 - 1 || yi < 1 || yi >= self.h as i64 - 1 {
        return false;
      }
      // Touch test: any of the 8 neighbours is ice?
      let touched = (-1..=1_i64).any(|dy| {
        (-1..=1_i64).any(|dx| {
          if dx == 0 && dy == 0 {
            return false;
          }
          let nx = xi + dx;
          let ny = yi + dy;
          self.ice[ny as usize * self.w + nx as usize]
        })
      });
      if touched {
        // Stickiness gate: maybe bounce instead of attaching.
        if self.rng.gen::<f64>() > m.stickiness {
          // Bounce back along the step direction.
          x -= step_angle.cos() / ax;
          y -= step_angle.sin();
          continue;
        }
        // Attach: freeze this position and its 5 rotational copies for
        // C6 symmetry.
        let local_dx_world = (x - cx) * ax;
        let local_dy_world = y - cy;
        self.freeze_six(local_dx_world, local_dy_world, ax, t);
        return true;
      }
    }
    false
  }

  /// Freeze the cell at (dx_world, dy_world) relative to centre, plus its
  /// 5 rotational copies (60° apart).
  fn freeze_six(&mut self, dx: f64, dy: f64, ax: f64, t: f64) {
    let cx = self.w as f64 * 0.5;
    let cy = self.h as f64 * 0.5;
    for k in 0..6 {
      let theta = TAU * (k as f64) / 6.0;
      let rdx = dx * theta.cos() - dy * theta.sin();
      let rdy = dx * theta.sin() + dy * theta.cos();
      let sx = (cx + rdx / ax).round() as i64;
      let sy = (cy + rdy).round() as i64;
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
      // Rewind or first frame: full reset.
      self.reseed(w, h);
    } else if w != self.w || h != self.h {
      // Resize: rebuild buffers and reseed (the previous crystal would not
      // fit cleanly into the new aspect).
      self.reseed(w, h);
    }
    let dt = (ctx.elapsed - self.last).clamp(0.0, 0.1);
    self.last = ctx.elapsed;
    let ax = w as f64 / (h as f64 * 2.0).max(1.0);
    let m = self.morphology();

    match self.phase {
      Phase::Growing => {
        for _ in 0..WALKERS_PER_FRAME {
          self.launch_walker(ctx.elapsed, ax);
        }
        if self.cell_count >= m.max_cells {
          self.phase = Phase::Holding;
          self.hold_until = ctx.elapsed + DISPLAY_SECONDS;
        }
      }
      Phase::Holding => {
        if ctx.elapsed >= self.hold_until {
          self.phase = Phase::Dissolving;
          self.dissolve_until = ctx.elapsed + 2.5;
        }
      }
      Phase::Dissolving => {
        let decay = (-dt * DISSOLVE_RATE).exp();
        for g in self.glow.iter_mut() {
          *g *= decay;
        }
        if ctx.elapsed >= self.dissolve_until {
          // Advance to the next crystal morphology and start growing again.
          self.morph_idx = self.morph_idx.wrapping_add(1);
          self.reseed(w, h);
        }
      }
    }

    let contrast = ctx.options.contrast;
    let mut grid = vec![0.0_f64; w * h];
    // Brightness during Growing/Holding: ice cells light up, freshly-frozen
    // a touch brighter so the growth front leads the eye. During Dissolving
    // we use the decaying glow buffer directly.
    let cx_f = w as f64 * 0.5;
    let cy_f = h as f64 * 0.5;
    let max_r = self.crystal_radius(ax).max(1.0);
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
              let freshness = (1.0 - recency / 1.5).clamp(0.0, 1.0);
              // Radial darkening so the centre reads as solid ice and the
              // tips as freshly-grown spikes.
              let dx = (col as f64 - cx_f) * ax;
              let dy = row as f64 - cy_f;
              let r = (dx * dx + dy * dy).sqrt() / max_r;
              let radial = (1.0 - 0.35 * r).clamp(0.3, 1.0);
              (0.65 + 0.30 * freshness) * radial
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
    let name = match self.morph_idx % MORPHOLOGIES.len() {
      0 => "stellar dendrite",
      1 => "fern dendrite",
      _ => "plate",
    };
    let phase = match self.phase {
      Phase::Growing => "growing",
      Phase::Holding => "holding",
      Phase::Dissolving => "dissolving",
    };
    Some(format!("{} | {} | {} cells", name, phase, self.cell_count))
  }
}
