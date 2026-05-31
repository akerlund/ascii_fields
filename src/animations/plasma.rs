use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, render_field, FieldStyle, DEFAULT_THRESHOLDS};

const STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "plasma" };

pub struct Plasma;

impl Animation for Plasma {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let w = ctx.width; let h = ctx.height;
    let mut grid = vec![0.0_f64; w * h];
    let t = ctx.elapsed;
    let freq = ctx.options.scale.max(0.4);
    let contrast = ctx.options.contrast;
    let cx = 0.5 + 0.3 * (t * 0.6).sin();
    let cy = 0.5 + 0.3 * (t * 0.5).cos();
    let dw = (w.saturating_sub(1)).max(1) as f64;
    let dh = (h.saturating_sub(1)).max(1) as f64;
    for row in 0..h {
      let v = row as f64 / dh;
      let y = v * 6.0 * freq;
      let base = row * w;
      for col in 0..w {
        let u = col as f64 / dw;
        let x = u * 6.0 * freq;
        let mut value = (x + t).sin();
        value += (y * 1.3 - t * 0.8).sin();
        value += ((x + y) * 0.7 + t * 0.5).sin();
        let dx = u - cx; let dy = v - cy;
        let d = (dx * dx + dy * dy).sqrt() * 10.0 * freq;
        value += (d - t * 1.6).sin();
        let level = 0.5 + 0.5 * value / 4.0;
        grid[base + col] = clamp(0.5 + (level - 0.5) * contrast);
      }
    }
    render_field(ctx, &grid, DEFAULT_THRESHOLDS, &STYLE, out);
  }
}
