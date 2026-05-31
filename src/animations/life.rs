use rand::{Rng, SeedableRng};
use rand_pcg::Pcg32;
use std::collections::HashMap;
use crate::animation::{Animation, FrameContext};
use crate::core::{render_field, FieldStyle};

const STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "aurora" };
const TH: &[(f64, char)] = &[
  (0.08, ' '), (0.20, '.'), (0.34, ':'), (0.48, '-'), (0.62, '='),
  (0.74, '+'), (0.85, '*'), (0.93, '#'), (1.01, '@'),
];
const STEP_DT: f64 = 0.11;
const RESEED_GEN: i64 = 900;

pub struct Life {
  w: usize, h: usize,
  cells: HashMap<(i64, i64), bool>,
  age: Vec<i32>,
  gen: i64,
  last_elapsed: f64,
  stable: i32,
  last_pop: i64,
  rng: Pcg32,
}

impl Default for Life {
  fn default() -> Self {
    Self {
      w: 0, h: 0, cells: HashMap::new(), age: Vec::new(), gen: 0,
      last_elapsed: 0.0, stable: 0, last_pop: -1,
      rng: Pcg32::from_entropy(),
    }
  }
}

impl Life {
  fn seed(&mut self, w: usize, h: usize) {
    self.w = w; self.h = h;
    self.cells.clear();
    self.age = vec![999; w * h];
    let fill = 0.30;
    for y in 0..h as i64 {
      for x in 0..w as i64 {
        if self.rng.gen::<f64>() < fill {
          self.cells.insert((x, y), true);
          self.age[(y as usize) * w + (x as usize)] = 0;
        }
      }
    }
    self.gen = 0;
    self.stable = 0;
    self.last_pop = -1;
  }
  fn step(&mut self) {
    let w = self.w as i64; let h = self.h as i64;
    let mut counts: HashMap<(i64, i64), i32> = HashMap::new();
    for &(x, y) in self.cells.keys() {
      for dy in -1..=1 {
        let ny = (y + dy).rem_euclid(h);
        for dx in -1..=1 {
          if dx == 0 && dy == 0 { continue; }
          let nx = (x + dx).rem_euclid(w);
          *counts.entry((nx, ny)).or_insert(0) += 1;
        }
      }
    }
    let mut new_cells: HashMap<(i64, i64), bool> = HashMap::new();
    for (cell, c) in &counts {
      if *c == 3 || (*c == 2 && self.cells.contains_key(cell)) {
        new_cells.insert(*cell, true);
      }
    }
    self.cells = new_cells;
    for a in self.age.iter_mut() { *a += 1; }
    for &(x, y) in self.cells.keys() {
      self.age[(y as usize) * self.w + (x as usize)] = 0;
    }
    self.gen += 1;
    let pop = self.cells.len() as i64;
    if pop == self.last_pop { self.stable += 1; } else { self.stable = 0; }
    self.last_pop = pop;
  }
}

impl Animation for Life {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    if self.age.is_empty() || ctx.width != self.w || ctx.height != self.h || ctx.elapsed < self.last_elapsed {
      self.seed(ctx.width, ctx.height);
    }
    self.last_elapsed = ctx.elapsed;
    let target = (ctx.elapsed / STEP_DT) as i64;
    let mut guard = 0;
    while self.gen < target && guard < 200 {
      self.step();
      guard += 1;
      if self.cells.is_empty() || self.stable > 40 || self.gen > RESEED_GEN {
        self.seed(ctx.width, ctx.height);
        self.gen = target;
      }
    }
    let mut grid = vec![0.0_f64; ctx.width * ctx.height];
    for r in 0..ctx.height {
      let base = r * ctx.width;
      for c in 0..ctx.width {
        let a = self.age[base + c];
        grid[base + c] = if a == 0 { 1.0 } else if a < 9 { 0.62 - 0.06 * a as f64 } else { 0.0 };
      }
    }
    render_field(ctx, &grid, TH, &STYLE, out);
  }
}
