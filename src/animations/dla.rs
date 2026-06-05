use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, render_field, FieldStyle};
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg32;

const STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "ice" };
const TH: &[(f64, char)] = &[
  (0.05, ' '),
  (0.20, '.'),
  (0.34, ':'),
  (0.48, '-'),
  (0.62, '='),
  (0.74, '+'),
  (0.85, '*'),
  (0.93, '#'),
  (1.01, '@'),
];
const MAX_WALK: i32 = 1500;

/// Two-phase lifecycle to avoid the "expanding then noise" problem the
/// continuous-decay version had: walkers attaching to fragmented dying
/// cells produced sparse scatter instead of a coherent dendrite.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
  /// Cluster is growing toward cap_size. Walkers attach as usual.
  Growing,
  /// Cluster has reached cap; oldest cells (deepest interior, near the
  /// seed) are removed each frame. No new walkers; the dendrite visibly
  /// melts from the inside out until very small, then we restart growth.
  Melting,
}

pub struct Dla {
  w: usize,
  h: usize,
  cluster: Vec<bool>,
  age: Vec<i32>,
  gen: i32,
  last: f64,
  phase: Phase,
  rng: Pcg32,
}
impl Default for Dla {
  fn default() -> Self {
    Self {
      w: 0,
      h: 0,
      cluster: Vec::new(),
      age: Vec::new(),
      gen: 0,
      last: 0.0,
      phase: Phase::Growing,
      rng: Pcg32::from_entropy(),
    }
  }
}

impl Dla {
  fn seed(&mut self, w: usize, h: usize) {
    self.w = w;
    self.h = h;
    self.cluster = vec![false; w * h];
    self.age = vec![0; w * h];
    let cx = w / 2;
    let cy = h / 2;
    self.cluster[cy * w + cx] = true;
    self.age[cy * w + cx] = 1;
    self.gen = 1;
    self.phase = Phase::Growing;
  }
  fn walk_one(&mut self) {
    let w = self.w;
    let h = self.h;
    let mut x = 0i64;
    let mut y = 0i64;
    for _ in 0..20 {
      x = self.rng.gen_range(0..w as i64);
      y = self.rng.gen_range(0..h as i64);
      if !self.cluster[(y as usize) * w + (x as usize)] {
        break;
      }
    }
    for _ in 0..MAX_WALK {
      let base = (y as usize) * w;
      for &(dx, dy) in &[(1i64, 0i64), (-1, 0), (0, 1), (0, -1)] {
        let nx = (x + dx).rem_euclid(w as i64);
        let ny = (y + dy).rem_euclid(h as i64);
        if self.cluster[(ny as usize) * w + (nx as usize)] {
          self.cluster[base + (x as usize)] = true;
          self.gen += 1;
          self.age[base + (x as usize)] = self.gen;
          return;
        }
      }
      let &(dx, dy) = &[(1i64, 0i64), (-1, 0), (0, 1), (0, -1)][self.rng.gen_range(0..4)];
      x = (x + dx).rem_euclid(w as i64);
      y = (y + dy).rem_euclid(h as i64);
    }
  }
}

impl Animation for Dla {
  fn status(&self) -> Option<String> {
    let size: usize = self.cluster.iter().filter(|&&b| b).count();
    let phase = match self.phase {
      Phase::Growing => "growing",
      Phase::Melting => "melting",
    };
    Some(format!("{} cells, {}", size, phase))
  }

  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    if self.cluster.is_empty() || ctx.width != self.w || ctx.height != self.h || ctx.elapsed < self.last {
      self.seed(ctx.width, ctx.height);
    }
    self.last = ctx.elapsed;
    let scale = ctx.options.scale.max(1.0) as usize;
    let walkers = (40.max(self.w * self.h / 40)) * scale;

    let cap = (self.w as f64 * self.h as f64 * 0.42) as usize;
    let restart_size = (self.w as f64 * self.h as f64 * 0.02) as usize;
    let size: usize = self.cluster.iter().filter(|&&b| b).count();

    match self.phase {
      Phase::Growing => {
        for _ in 0..walkers {
          self.walk_one();
        }
        if size >= cap {
          self.phase = Phase::Melting;
        }
      }
      Phase::Melting => {
        // Remove the oldest cells first (interior, deepest in the
        // dendrite) so the structure visibly melts from the seed
        // outward, peeling back the branches.
        //
        // Death rate: ~3% of current size per frame so the melt lasts a
        // couple of seconds rather than blinking off in one step.
        let to_kill = (size / 35).max(60);
        let mut targets: Vec<(i32, usize)> = self
          .cluster
          .iter()
          .enumerate()
          .filter(|(_, &alive)| alive)
          .map(|(idx, _)| (self.age[idx], idx))
          .collect();
        targets.sort_by_key(|(age, _)| *age);
        for (_, idx) in targets.into_iter().take(to_kill) {
          self.cluster[idx] = false;
          self.age[idx] = 0;
        }
        if size <= restart_size {
          // Re-seed from a new random point on the screen so successive
          // dendrites look different from each other.
          let mut x = self.rng.gen_range(0..self.w);
          let mut y = self.rng.gen_range(0..self.h);
          if x.abs_diff(self.w / 2) < 4 && y.abs_diff(self.h / 2) < 4 {
            // Steer the new seed away from where the previous one was so
            // consecutive dendrites do not just retrace.
            x = (x + self.w / 3) % self.w;
            y = (y + self.h / 3) % self.h;
          }
          let idx = y * self.w + x;
          self.cluster.iter_mut().for_each(|b| *b = false);
          self.age.iter_mut().for_each(|a| *a = 0);
          self.cluster[idx] = true;
          self.gen += 1;
          self.age[idx] = self.gen;
          self.phase = Phase::Growing;
        }
      }
    }

    // Brightness: freshness within the current age window so the newest
    // branches glow brightest and the trunk dims toward the seed.
    let max_age = self.gen as f64;
    let min_age = (self.gen - (cap as i32 * 8).max(2000)).max(0) as f64;
    let span = (max_age - min_age).max(1.0);
    let contrast = ctx.options.contrast;
    let mut grid = vec![0.0_f64; ctx.width * ctx.height];
    for r in 0..ctx.height {
      let base = r * ctx.width;
      for c in 0..ctx.width {
        let a = self.age[base + c];
        if a > 0 {
          let freshness = ((a as f64 - min_age) / span).clamp(0.0, 1.0);
          grid[base + c] = clamp((0.18 + 0.78 * freshness) * contrast);
        }
      }
    }
    render_field(ctx, &grid, TH, &STYLE, out);
  }
}
