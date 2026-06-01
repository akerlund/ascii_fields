use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, render_field, FieldStyle};

const STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "ocean" };
const TH: &[(f64, char)] = &[
  (0.10, ' '),
  (0.22, '.'),
  (0.34, ':'),
  (0.46, '-'),
  (0.58, '='),
  (0.70, '+'),
  (0.82, '*'),
  (0.92, '#'),
  (1.01, '@'),
];
const SHED_DT: f64 = 0.95;
const FLOW_SPEED: f64 = 0.55;
const OBSTACLE_X: f64 = -0.55;
const OBSTACLE_R: f64 = 0.10;
const OFFSET: f64 = 0.16;

pub struct Karman {
  vortices: Vec<(f64, f64)>, // (sign, t0)
  next_shed: f64,
  last: f64,
  even: bool,
}
impl Default for Karman {
  fn default() -> Self {
    Self { vortices: Vec::new(), next_shed: 0.0, last: 0.0, even: false }
  }
}

impl Animation for Karman {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    if ctx.elapsed < self.last {
      self.vortices.clear();
      self.next_shed = 0.0;
      self.even = false;
    }
    self.last = ctx.elapsed;
    while ctx.elapsed >= self.next_shed {
      let sign = if self.even { 1.0 } else { -1.0 };
      self.vortices.push((sign, self.next_shed));
      self.even = !self.even;
      self.next_shed += SHED_DT;
    }
    self.vortices.retain(|&(_, t0)| ctx.elapsed - t0 < 10.0);
    let ax = ctx.width as f64 / (ctx.height as f64 * 2.0).max(1.0);
    let contrast = ctx.options.contrast;
    let mut grid = vec![0.0_f64; ctx.width * ctx.height];
    let dw = (ctx.width.saturating_sub(1)).max(1) as f64;
    let dh = (ctx.height.saturating_sub(1)).max(1) as f64;
    for row in 0..ctx.height {
      let py = (row as f64 / dh - 0.5) * 2.0;
      let base = row * ctx.width;
      for col in 0..ctx.width {
        let px = (col as f64 / dw - 0.5) * 2.0 * ax;
        let dx_obs = px - OBSTACLE_X;
        let d_obs = (dx_obs * dx_obs + py * py).sqrt();
        if d_obs < OBSTACLE_R {
          continue;
        }
        let bg = 0.10 + 0.04 * (8.0 * py + 0.6 * ctx.elapsed).sin();
        let mut accum = 0.0_f64;
        for &(sign, t0) in &self.vortices {
          let age = ctx.elapsed - t0;
          let vx = OBSTACLE_X + OBSTACLE_R + FLOW_SPEED * age;
          let vy = sign * OFFSET + sign * 0.04 * (1.6 * age).sin();
          let r2 = (px - vx).powi(2) + (py - vy).powi(2);
          let sigma2 = 0.0030 + 0.0035 * age;
          accum += sign * (-r2 / (2.0 * sigma2)).exp() * (-age * 0.16).exp() * 0.9;
        }
        grid[base + col] = clamp((bg + accum.abs() * 1.4) * contrast);
      }
    }
    render_field(ctx, &grid, TH, &STYLE, out);
  }
}
