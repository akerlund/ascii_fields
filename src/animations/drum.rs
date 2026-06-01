use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, render_field, FieldStyle};
use std::collections::HashMap;

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
// (n angular nodes, m radial index, z_{n,m})
const MODES: &[(usize, usize, f64)] = &[
  (0, 1, 2.4048),
  (1, 1, 3.8317),
  (2, 1, 5.1356),
  (0, 2, 5.5201),
  (3, 1, 6.3802),
  (1, 2, 7.0156),
  (4, 1, 7.5883),
  (2, 2, 8.4172),
  (0, 3, 8.6537),
];
const MODE_SECONDS: f64 = 4.0;
const RADIAL_SAMPLES: usize = 256;

fn factorial(n: usize) -> f64 {
  let mut f = 1.0_f64;
  for i in 1..=n {
    f *= i as f64;
  }
  f
}

fn bessel_j(n: usize, x: f64) -> f64 {
  if x == 0.0 {
    return if n == 0 { 1.0 } else { 0.0 };
  }
  let half = x * 0.5;
  let mut term = half.powi(n as i32) / factorial(n);
  let mut total = term;
  let mut sign = -1.0_f64;
  let mut k = 1;
  while k < 80 {
    term *= (half * half) / (k as f64 * (k + n) as f64);
    let delta = sign * term;
    total += delta;
    if delta.abs() < 1e-9 {
      break;
    }
    sign = -sign;
    k += 1;
  }
  total
}

pub struct Drum {
  profile: HashMap<(usize, u64), Vec<f64>>,
}
impl Default for Drum {
  fn default() -> Self {
    Self { profile: HashMap::new() }
  }
}

impl Drum {
  fn radial(&mut self, n: usize, zero: f64) -> &Vec<f64> {
    let key = (n, zero.to_bits());
    self.profile.entry(key).or_insert_with(|| {
      let denom = (RADIAL_SAMPLES - 1) as f64;
      let mut prof: Vec<f64> = (0..RADIAL_SAMPLES).map(|i| bessel_j(n, zero * i as f64 / denom)).collect();
      let peak = prof.iter().map(|v| v.abs()).fold(0.0_f64, f64::max).max(1.0);
      for v in prof.iter_mut() {
        *v /= peak;
      }
      prof
    })
  }
}

impl Animation for Drum {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let idx = ((ctx.elapsed / MODE_SECONDS) as usize) % MODES.len();
    let (n, _m, zero) = MODES[idx];
    let omega = zero.sqrt() * 0.9;
    let pulse = 0.72 + 0.28 * (omega * ctx.elapsed).cos();
    let ax = ctx.width as f64 / (ctx.height as f64 * 2.0).max(1.0);
    let contrast = ctx.options.contrast;
    let last_idx = (RADIAL_SAMPLES - 1) as f64;
    let prof = self.radial(n, zero).clone();
    let mut grid = vec![0.0_f64; ctx.width * ctx.height];
    let dw = (ctx.width.saturating_sub(1)).max(1) as f64;
    let dh = (ctx.height.saturating_sub(1)).max(1) as f64;
    for row in 0..ctx.height {
      let py = (row as f64 / dh - 0.5) * 2.0;
      let base = row * ctx.width;
      for col in 0..ctx.width {
        let px = (col as f64 / dw - 0.5) * 2.0 * ax;
        let r = (px * px + py * py).sqrt();
        if r >= 1.0 {
          continue;
        }
        let ri = (r * last_idx) as usize;
        let radial = prof[ri.min(RADIAL_SAMPLES - 1)];
        let u = radial * (n as f64 * py.atan2(px)).cos();
        grid[base + col] = clamp(u.abs() * pulse * 1.4 * contrast);
      }
    }
    render_field(ctx, &grid, TH, &STYLE, out);
  }
}
