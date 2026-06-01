use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, render_field, FieldStyle};

const STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "amber" };
const TH: &[(f64, char)] = &[
  (0.05, ' '),
  (0.18, '.'),
  (0.32, ':'),
  (0.46, '-'),
  (0.60, '='),
  (0.72, '+'),
  (0.84, '*'),
  (0.93, '#'),
  (1.01, '@'),
];
const SIGMA: f64 = 10.0;
const RHO: f64 = 28.0;
const BETA: f64 = 8.0 / 3.0;
const DT: f64 = 0.012;
const STEPS_PER_FRAME: i32 = 50;
const TRAIL_DECAY: f64 = 0.94;

pub struct Lorenz {
  w: usize,
  h: usize,
  grid: Vec<f64>,
  x: f64,
  y: f64,
  z: f64,
  last: f64,
}
impl Default for Lorenz {
  fn default() -> Self {
    Self { w: 0, h: 0, grid: Vec::new(), x: 0.1, y: 0.0, z: 0.0, last: 0.0 }
  }
}

impl Lorenz {
  fn seed(&mut self, w: usize, h: usize) {
    self.w = w;
    self.h = h;
    self.grid = vec![0.0; w * h];
    self.x = 0.1;
    self.y = 0.0;
    self.z = 0.0;
  }
}

impl Animation for Lorenz {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    if self.grid.is_empty() || ctx.width != self.w || ctx.height != self.h || ctx.elapsed < self.last {
      self.seed(ctx.width, ctx.height);
    }
    self.last = ctx.elapsed;
    let scale = ctx.options.scale.max(0.4);
    let decay = TRAIL_DECAY.powf(scale.max(0.5));
    for v in self.grid.iter_mut() {
      *v *= decay;
    }
    let fx = ctx.width as f64 * 0.5 / 30.0;
    let fy = ctx.height as f64 * 0.5 / 30.0;
    let cx = ctx.width as f64 * 0.5;
    let cy = ctx.height as f64 * 0.5;
    let steps = (STEPS_PER_FRAME as f64 * scale) as i32;
    let mut x = self.x;
    let mut y = self.y;
    let mut z = self.z;
    for _ in 0..steps.max(1) {
      let dx = SIGMA * (y - x);
      let dy = x * (RHO - z) - y;
      let dz = x * y - BETA * z;
      x += dx * DT;
      y += dy * DT;
      z += dz * DT;
      let ci = (cx + x * fx) as i64;
      let ri = (cy + (z - 25.0) * fy) as i64;
      if ci >= 0 && (ci as usize) < ctx.width && ri >= 0 && (ri as usize) < ctx.height {
        let idx = (ri as usize) * ctx.width + (ci as usize);
        if self.grid[idx] < 1.0 {
          self.grid[idx] = (self.grid[idx] + 0.55).min(1.0);
        }
      }
    }
    self.x = x;
    self.y = y;
    self.z = z;
    let contrast = ctx.options.contrast;
    let mut out_grid = vec![0.0_f64; ctx.width * ctx.height];
    for i in 0..out_grid.len() {
      out_grid[i] = clamp(self.grid[i] * contrast);
    }
    render_field(ctx, &out_grid, TH, &STYLE, out);
  }
}
