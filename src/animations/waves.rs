use std::f64::consts::PI;
use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, render_field, smoothstep, FieldStyle};

const STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "ocean" };
const TH: &[(f64, char)] = &[
  (0.16, ' '), (0.24, '.'), (0.33, ':'), (0.44, '-'), (0.55, '='),
  (0.68, '+'), (0.82, '*'), (0.94, '#'), (1.01, '%'),
];

pub struct Waves;

impl Animation for Waves {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let w = ctx.width; let h = ctx.height;
    let mut grid = vec![0.0_f64; w * h];
    let density = ctx.options.scale.max(0.45);
    let t = ctx.elapsed;
    let dw = (w.saturating_sub(1)).max(1) as f64;
    let dh = (h.saturating_sub(1)).max(1) as f64;
    let contrast = ctx.options.contrast;
    for row in 0..h {
      let y = row as f64 / dh;
      let vy = (y - 0.5).abs() * 2.0;
      let base = row * w;
      for col in 0..w {
        let x = col as f64 / dw;
        let mut shore = 0.70;
        shore += 0.10 * (2.0 * PI * x + 0.20 * (t * 0.19).sin()).sin();
        shore += 0.055 * (5.4 * PI * x - t * 0.11).sin();
        shore += 0.025 * (12.0 * PI * x + 0.13 * t).sin();
        let water = 1.0 - smoothstep(shore - 0.045, shore + 0.020, y);
        let beach = 1.0 - water;
        let depth = (shore - y).max(0.0);
        let shallow = 1.0 - smoothstep(0.02, 0.48, depth);
        let shoal_gain = 0.28 + 0.90 * shallow;
        let mut sandbar = 0.05 * (3.0 * PI * x - 0.23 * t).sin();
        sandbar += 0.025 * (9.0 * PI * x + 0.31 * t).sin();
        let refr = (depth + sandbar * shallow).max(0.0);
        let pa = 0.62 + 0.38 * (1.4 * x + 0.31 * t).sin();
        let pb = 0.64 + 0.36 * (3.6 * x - 0.17 * t + 1.2).sin();
        let aa = (18.0 * density) * refr + 1.8 * (2.4 * PI * x + 0.16 * t).sin();
        let ab = (11.0 * density) * (refr + 0.06 * (5.0 * x - 0.2 * t).sin());
        let ca = smoothstep(0.78, 0.985, (aa - 2.7 * t).sin());
        let cb = smoothstep(0.80, 0.990, (ab - 1.6 * t + 1.4).sin());
        let incoming = water * shoal_gain * (0.54 * pa * ca + 0.34 * pb * cb);
        let back_axis = depth + 0.10 * (4.0 * PI * x + 0.2 * t).sin();
        let mut backwash = smoothstep(0.80, 0.99, ((13.0 * density) * back_axis + 1.85 * t).sin());
        backwash *= (0.20 + 0.80 * shallow) * (0.65 + 0.35 * (2.0 * x - 0.41 * t).sin());
        let breaker = (-42.0 * depth).exp();
        let foam_spread = smoothstep(0.18, 0.02, depth);
        let shore_foam = water * breaker * (0.35 + 0.65 * ca.max(cb));
        let mut wash_up = beach * (-28.0 * (y - shore).max(0.0)).exp();
        wash_up *= 0.35 + 0.65 * smoothstep(0.25, 0.95, (8.0 * (y - shore) - 1.35 * t).sin());
        let ripples = 0.06 * water * (1.0 - shallow) * (38.0 * depth + 6.0 * x - 1.1 * t).sin().abs();
        let wet_sand = beach * (0.10 + 0.16 * (-16.0 * (y - shore).max(0.0)).exp());
        let mut texture = incoming + 0.42 * backwash + 0.35 * foam_spread * ca.max(cb);
        texture += shore_foam + wash_up + ripples + wet_sand;
        texture = texture.min(1.0);
        let vx = (x - 0.5).abs() * 2.0;
        let vignette = (1.0 - 0.18 * vx * vx - 0.10 * vy * vy).max(0.0);
        grid[base + col] = clamp(texture * vignette * contrast);
      }
    }
    render_field(ctx, &grid, TH, &STYLE, out);
  }
}
