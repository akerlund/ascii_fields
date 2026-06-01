use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, render_field, FieldStyle};

const STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "amber" };
const TH: &[(f64, char)] = &[
  (0.06, ' '),
  (0.22, '.'),
  (0.36, ':'),
  (0.50, '-'),
  (0.62, '='),
  (0.74, '+'),
  (0.84, '*'),
  (0.93, '#'),
  (1.01, '@'),
];
const MAX_BITS: i32 = 15;
const SPEED: f64 = 0.32;

fn membership(mut x: f64, mut y: f64) -> i32 {
  for i in 0..MAX_BITS {
    x *= 2.0;
    y *= 2.0;
    let bx = x >= 1.0;
    let by = y >= 1.0;
    if bx {
      x -= 1.0;
    }
    if by {
      y -= 1.0;
    }
    if bx && by {
      return i;
    }
  }
  MAX_BITS
}

pub struct Sierpinski;

impl Animation for Sierpinski {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let frac = (ctx.elapsed * SPEED) % 1.0;
    let window = 2.0_f64.powf(-frac);
    let contrast = ctx.options.contrast;
    let w = ctx.width;
    let h = ctx.height;
    let mut grid = vec![0.0_f64; w * h];
    let dw = (w.saturating_sub(1)).max(1) as f64;
    let dh = (h.saturating_sub(1)).max(1) as f64;
    for row in 0..h {
      let y = (1.0 - row as f64 / dh) * window;
      let base = row * w;
      for col in 0..w {
        let x = (col as f64 / dw) * window;
        let depth = membership(x, y);
        let level = depth as f64 / MAX_BITS as f64;
        grid[base + col] = clamp(level * contrast);
      }
    }
    render_field(ctx, &grid, TH, &STYLE, out);
  }
}
