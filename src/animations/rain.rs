use rand::{Rng, SeedableRng};
use rand_pcg::Pcg32;
use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, render_glyph_field, FieldStyle};

const STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "aurora" };
const GLYPHS: &str = "abcdefghijklmnopqrstuvwxyz0123456789@#$%&*+=<>|/\\:;.-";

pub struct Rain {
  w: usize, h: usize,
  offsets: Vec<f64>,
  speeds: Vec<f64>,
  lengths: Vec<i64>,
  rng: Pcg32,
}
impl Default for Rain {
  fn default() -> Self {
    Self { w: 0, h: 0, offsets: Vec::new(), speeds: Vec::new(), lengths: Vec::new(), rng: Pcg32::from_entropy() }
  }
}

impl Rain {
  fn seed(&mut self, w: usize, h: usize) {
    self.w = w; self.h = h;
    self.offsets = (0..w).map(|_| self.rng.gen_range(0.0..h as f64)).collect();
    self.speeds = (0..w).map(|_| self.rng.gen_range(6.0..18.0)).collect();
    let lo = 5_i64; let hi = (6_i64).max(h as i64 / 2);
    self.lengths = (0..w).map(|_| self.rng.gen_range(lo..=hi)).collect();
  }
}

impl Animation for Rain {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    if self.offsets.is_empty() || ctx.width != self.w || ctx.height != self.h {
      self.seed(ctx.width, ctx.height);
    }
    let speed_mul = ctx.options.scale.max(0.4);
    let w = ctx.width; let h = ctx.height;
    let chars: Vec<char> = GLYPHS.chars().collect();
    let n = chars.len();
    let mut heads = vec![0.0_f64; w];
    for c in 0..w {
      heads[c] = ((self.offsets[c] + ctx.elapsed * self.speeds[c] * speed_mul)
                  % (h as f64 + self.lengths[c] as f64) + h as f64 + self.lengths[c] as f64)
                  % (h as f64 + self.lengths[c] as f64);
    }
    let flick = (ctx.elapsed * 12.0) as i64;
    let mut grid = vec![0.0_f64; w * h];
    let mut glyphs = vec![' '; w * h];
    for row in 0..h {
      for col in 0..w {
        let head = heads[col];
        let d = head - row as f64;
        let length = self.lengths[col];
        let (level, glyph) = if d >= 0.0 && d < length as f64 {
          let level = if d < 1.0 { 1.0 } else { clamp((1.0 - d / length as f64) * 0.85) };
          let f = if d < 2.0 { flick } else { flick / 6 };
          let gi = (((col * 31 + row * 17) as i64 + f) % n as i64).rem_euclid(n as i64) as usize;
          (level, chars[gi])
        } else { (0.0, ' ') };
        grid[row * w + col] = level;
        glyphs[row * w + col] = glyph;
      }
    }
    render_glyph_field(ctx, &grid, &glyphs, &STYLE, out);
  }
}
