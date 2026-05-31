use crate::animation::{Animation, FrameContext};
use crate::core::{render_glyph_field, FieldStyle};

const STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "ocean" };

pub struct Dna;

impl Animation for Dna {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let w = ctx.width; let h = ctx.height;
    let cx = w as f64 * 0.5;
    let amp = (w as f64 * 0.20).max(3.0);
    let twist = 0.36 * ctx.options.scale.max(0.4);
    let t = ctx.elapsed * 1.8;
    let mut grid = vec![0.0_f64; w * h];
    let mut glyphs = vec![' '; w * h];
    let put = |grid: &mut Vec<f64>, glyphs: &mut Vec<char>, col: f64, row: usize, value: f64, ch: char| {
      let c = col.round() as i64;
      if c >= 0 && (c as usize) < w && row < h {
        let idx = row * w + (c as usize);
        if grid[idx] < value { grid[idx] = value; glyphs[idx] = ch; }
      }
    };
    // rungs
    for row in 0..h {
      let p = row as f64 * twist + t;
      let s = p.sin();
      if s.abs() > 0.18 {
        let xa = cx + amp * s;
        let xb = cx - amp * s;
        let (x0, x1) = if xa < xb { (xa, xb) } else { (xb, xa) };
        let spread = s.abs();
        let rung_b = 0.30 + 0.20 * spread;
        let steps = ((x1 - x0) as i64).max(2);
        for i in 1..steps {
          let glyph = if i % 2 == 0 { '=' } else { '-' };
          put(&mut grid, &mut glyphs, x0 + i as f64, row, rung_b, glyph);
        }
      }
    }
    // strands
    for &sign in &[1.0, -1.0] {
      let mut prev_x: Option<f64> = None;
      for row in 0..h {
        let p = row as f64 * twist + t;
        let s = p.sin(); let c = p.cos();
        let x = cx + sign * amp * s;
        let depth = c * sign;
        let slope = sign * amp * c * twist;
        let bright = 0.55 + 0.45 * (depth * 0.5 + 0.5);
        let glyph = if slope.abs() < 0.35 { '|' } else if slope > 0.0 { '\\' } else { '/' };
        put(&mut grid, &mut glyphs, x, row, bright, glyph);
        put(&mut grid, &mut glyphs, x - 1.0, row, bright * 0.55, glyph);
        put(&mut grid, &mut glyphs, x + 1.0, row, bright * 0.55, glyph);
        if let Some(px) = prev_x {
          if (x - px).abs() > 1.4 {
            let step: i64 = if x < px { -1 } else { 1 };
            let mut fill = px as i64 + step;
            while fill != x as i64 {
              put(&mut grid, &mut glyphs, fill as f64, row, bright * 0.45, glyph);
              fill += step;
            }
          }
        }
        prev_x = Some(x);
      }
    }
    render_glyph_field(ctx, &grid, &glyphs, &STYLE, out);
  }
}
