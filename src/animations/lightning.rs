use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, render_field, FieldStyle};
use crate::noise::fbm;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg32;

const STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "ice" };
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
const STRIKE_PERIOD: f64 = 2.6;
const STRIKE_DURATION: f64 = 0.55;

pub struct Lightning {
  strike_idx: Option<i64>,
  centers: Vec<Vec<f64>>,
  w: usize,
  h: usize,
}
impl Default for Lightning {
  fn default() -> Self {
    Self { strike_idx: None, centers: Vec::new(), w: 0, h: 0 }
  }
}

impl Lightning {
  fn build_bolt(&mut self, seed: i64, width: usize, height: usize) {
    let mut rng = Pcg32::seed_from_u64((seed as u64).wrapping_mul(2654435761));
    let mut centers: Vec<Vec<f64>> = (0..height).map(|_| Vec::new()).collect();
    let mut x: f64 = rng.gen_range(width as f64 * 0.25..width as f64 * 0.75);
    let mut main = Vec::with_capacity(height);
    for row in 0..height {
      x += rng.gen_range(-1.6..1.6);
      if rng.gen::<f64>() < 0.12 {
        x += rng.gen_range(-4.0..4.0);
      }
      x = x.clamp(0.0, width as f64 - 1.0);
      main.push(x);
      centers[row].push(x);
    }
    let n_branches = if height >= 6 { rng.gen_range(2..=4) } else { 0 };
    for _ in 0..n_branches {
      let start = rng.gen_range(1..(height - 1)) as usize;
      let mut bx = main[start];
      let direction: f64 = if rng.gen::<bool>() { -1.0 } else { 1.0 };
      let length = rng.gen_range(3..(4_usize.max(height / 3) + 1));
      for k in 0..length {
        let row = start + k;
        if row >= height {
          break;
        }
        bx += direction * rng.gen_range(0.8..2.2) + rng.gen_range(-0.6..0.6);
        bx = bx.clamp(0.0, width as f64 - 1.0);
        centers[row].push(bx);
      }
    }
    self.centers = centers;
  }
  fn intensity(local: f64) -> f64 {
    if local > STRIKE_DURATION {
      return 0.0;
    }
    let mut f = (-local / 0.10).exp();
    f += 0.6 * (-((local - 0.16).powi(2)) / 0.0015).exp();
    f += 0.4 * (-((local - 0.30).powi(2)) / 0.002).exp();
    f.min(1.0)
  }
}

impl Animation for Lightning {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let idx = (ctx.elapsed / STRIKE_PERIOD) as i64;
    let local = ctx.elapsed - idx as f64 * STRIKE_PERIOD;
    if self.strike_idx != Some(idx) || ctx.width != self.w || ctx.height != self.h {
      self.strike_idx = Some(idx);
      self.w = ctx.width;
      self.h = ctx.height;
      self.build_bolt(idx, ctx.width, ctx.height);
    }
    let intensity = Self::intensity(local);
    let sky_flash = 0.28 * intensity;
    let contrast = ctx.options.contrast;
    let mut grid = vec![0.0_f64; ctx.width * ctx.height];
    let dw = (ctx.width.saturating_sub(1)).max(1) as f64;
    let dh = (ctx.height.saturating_sub(1)).max(1) as f64;
    for row in 0..ctx.height {
      let v = row as f64 / dh;
      let centers = &self.centers[row];
      let base = row * ctx.width;
      for col in 0..ctx.width {
        let u = col as f64 / dw;
        let clouds = 0.10 + 0.20 * fbm(u * 4.0 + ctx.elapsed * 0.05, v * 2.0, 4) * (1.0 - v * 0.5);
        let rain = 0.05 * (40.0 * (v + u * 0.5) - ctx.elapsed * 9.0).sin().max(0.0);
        let mut level = clouds + rain + sky_flash * (0.6 + 0.4 * (1.0 - v));
        if intensity > 0.0 && !centers.is_empty() {
          let mut glow = 0.0_f64;
          for &cxr in centers {
            let dx = col as f64 - cxr;
            let g = (-(dx * dx) / 1.6).exp();
            if g > glow {
              glow = g;
            }
          }
          level += glow * (0.4 + 0.6 * intensity) * 1.4;
        }
        grid[base + col] = clamp(level * contrast);
      }
    }
    render_field(ctx, &grid, TH, &STYLE, out);
  }
}
