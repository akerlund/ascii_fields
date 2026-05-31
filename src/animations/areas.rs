use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, render_field, smoothstep, FieldStyle};

const STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "amber" };
const TH: &[(f64, char)] = &[
  (0.20, ' '), (0.28, '.'), (0.36, ':'), (0.47, '-'), (0.58, '='),
  (0.70, '+'), (0.82, '*'), (0.93, '#'), (1.01, '%'),
];

pub struct Areas;

impl Animation for Areas {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let w = ctx.width; let h = ctx.height;
    let mut grid = vec![0.0_f64; w * h];
    let rx = (w as f64 * 0.48).max(1.0);
    let ry = (h as f64 * 0.43).max(1.0);
    let density = (ctx.options.scale * 0.62).max(0.35);
    let t = ctx.elapsed * 1.55;
    let sin_t = t.sin();
    let drift_x = 0.24 * sin_t + 0.10 * (ctx.elapsed * 0.47).sin();
    let drift_y = 0.11 * (t * 0.83).cos() + 0.06 * (ctx.elapsed * 0.31).sin();
    let contrast = ctx.options.contrast;
    let col_step = if w >= 40 { 2 } else { 1 };
    // pre-compute px per col-step group
    let mut cols: Vec<(usize, f64)> = Vec::new(); // (repeat, px)
    let mut c = 0;
    while c < w {
      let rep = col_step.min(w - c);
      let px = ((c as f64 + (col_step as f64 - 1.0) * 0.5) - (w as f64 - 1.0) * 0.5) / rx;
      cols.push((rep, px));
      c += col_step;
    }
    for row in 0..h {
      let py = (row as f64 - (h as f64 - 1.0) * 0.5) / ry;
      let py2 = py * py;
      let y = (py + drift_y) * density;
      let sin_y2 = (1.25 * y + 0.17 * t).sin();
      let sin_y27 = (1.85 * y + 0.42 * t).sin();
      let sin_y31 = (18.0 * y).sin();
      let mut col_out = 0;
      let base = row * w;
      for &(rep, px) in &cols {
        let r2 = px * px + py2;
        let level = if r2 >= 1.0 { 0.0 } else {
          let x = (px * 1.02 + drift_x) * density;
          let columns_a = smoothstep(0.42, 0.88, (4.5 * x + 1.1 * sin_y27).sin().abs());
          let columns_b = smoothstep(0.50, 0.94, (7.0 * x - 1.15 * y + 0.38 * t).sin().abs());
          let broad_a = smoothstep(-0.58, 0.50,
            sin_y2 + 0.72 * (1.65 * x - 0.95 * y + 0.50 * t).sin()
            + 0.48 * (2.35 * x + 0.45 * y - 0.25 * t).cos());
          let broad_b = smoothstep(-0.62, 0.56,
            (1.15 * x + 1.55 * y - 0.68 * t).sin()
            + 0.54 * (2.05 * x - 0.55 * y + 0.43 * t).cos());
          let broad_c = smoothstep(-0.50, 0.64,
            (0.85 * x - 1.95 * y + 0.34 * t).cos()
            + 0.46 * (2.70 * x + 0.25 * y - 0.58 * t).sin());
          let broad = (0.68 * broad_a + 0.46 * broad_b + 0.36 * broad_c).min(1.0);
          let holes = smoothstep(0.48, 0.92, (3.4 * x - 3.0 * y + 0.65 * sin_t).sin().abs());
          let stripes = smoothstep(0.22, 0.84, (sin_y31 + 0.18 * (2.2 * x).sin()).abs());
          let region = broad * (0.45 + 0.45 * columns_a.max(columns_b)) * (1.0 - 0.45 * holes);
          let texture = (0.16 + 0.07 * columns_a + 0.05 * stripes
            + region * (0.38 + 0.24 * stripes + 0.18 * columns_b)).min(1.0);
          let edge_fade = (1.0 - r2 * r2).max(0.0);
          let shadow = 0.62 + 0.38 * (1.0 - r2).max(0.0).sqrt();
          clamp(texture * edge_fade * shadow * contrast)
        };
        for _ in 0..rep {
          if col_out < w { grid[base + col_out] = level; }
          col_out += 1;
        }
      }
    }
    render_field(ctx, &grid, TH, &STYLE, out);
  }
}
