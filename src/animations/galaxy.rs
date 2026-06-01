use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, render_field, smoothstep, FieldStyle};
use crate::noise::star_noise;
use std::f64::consts::PI;

const STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "nebula" };
const TH: &[(f64, char)] = &[
  (0.11, ' '),
  (0.19, '.'),
  (0.29, ':'),
  (0.40, '-'),
  (0.52, '='),
  (0.65, '+'),
  (0.78, '*'),
  (0.91, '#'),
  (1.01, '%'),
];

pub struct Galaxy;

impl Animation for Galaxy {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let w = ctx.width;
    let h = ctx.height;
    let mut grid = vec![0.0_f64; w * h];
    let radius = (w.min(h * 2) as f64 * 0.47).max(1.0);
    let t = ctx.elapsed * 0.18;
    let contrast = ctx.options.contrast;
    for row in 0..h {
      let py = ((row as f64 - (h as f64 - 1.0) * 0.5) * 2.0) / radius;
      let tilted_y = py / 0.62;
      let base = row * w;
      for col in 0..w {
        let px = (col as f64 - (w as f64 - 1.0) * 0.5) / radius;
        let spin = t * (0.45 + 0.55 / (1.0 + 10.0 * (px * px + tilted_y * tilted_y)));
        let cs = spin.cos();
        let sn = spin.sin();
        let gx = px * cs - tilted_y * sn;
        let gy = px * sn + tilted_y * cs;
        let r = (gx * gx + gy * gy).sqrt();
        let theta = gy.atan2(gx);
        let core = (-18.0 * r * r).exp();
        let halo = (-2.8 * r).exp();
        let disk = smoothstep(1.18, 0.08, r);
        let twist = theta + 4.4 * (1.0 + r * 3.4).ln() - t * 1.35;
        let arm_a = smoothstep(0.84, 0.985, (2.0 * twist).cos());
        let arm_b = smoothstep(0.80, 0.990, (2.0 * twist + PI).cos());
        let arm_c = smoothstep(0.86, 0.995, (3.0 * twist - 0.6).cos());
        let arms = arm_a.max(0.82 * arm_b).max(0.52 * arm_c);
        let arm_fade = smoothstep(0.03, 0.18, r) * smoothstep(1.16, 0.42, r);
        let dust = smoothstep(
          0.20,
          0.86,
          (14.0 * theta + 9.0 * r - 0.7 * t).sin().abs() + 0.18 * (23.0 * r + 3.0 * theta).sin(),
        );
        let mottled = 0.70 + 0.30 * (33.0 * r + 7.0 * theta + t).sin().abs();
        let mut stars = 0.0_f64;
        if star_noise(col as i64, row as i64) > 0.993 {
          stars = 0.65 + 0.35 * (ctx.elapsed * 2.1 + col as f64 * 0.7).sin();
        } else if star_noise(col as i64 + 19, row as i64 - 7) > 0.985 {
          stars = 0.20;
        }
        let mut texture = 0.54 * core + 0.18 * halo * disk;
        texture += 0.62 * arms * arm_fade * mottled * (1.0 - 0.40 * dust);
        texture += stars * (1.0 - smoothstep(0.0, 0.45, core));
        let vignette = (1.0 - 0.10 * px.abs() - 0.16 * py.abs()).max(0.0);
        grid[base + col] = clamp(texture * vignette * contrast);
      }
    }
    render_field(ctx, &grid, TH, &STYLE, out);
  }
}
