//! Wave-plane: the original cyclic wave with a long scene-specific ASCII ramp.

use std::f64::consts::PI;
use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, render_field, shade, FieldStyle};

const STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "ocean" };

// (amplitude, wave_x, wave_y, time_speed, offset)
const WAVES: &[(f64, f64, f64, f64, f64)] = &[
  (0.26, 1.0, 2.0, 1.0, 0.00),
  (0.21, 2.0, -1.0, -1.0, 0.23),
  (0.18, 3.0, 1.0, 2.0, 0.41),
  (0.15, -2.0, 3.0, -2.0, 0.67),
  (0.13, 4.0, -3.0, 3.0, 0.11),
  (0.10, 5.0, 2.0, -3.0, 0.52),
  (0.08, -4.0, -5.0, 4.0, 0.31),
  (0.06, 7.0, -2.0, -4.0, 0.79),
  (0.05, 6.0, 5.0, 5.0, 0.18),
];

const RAMP_CLEAN: &str = " .'`^\",:;Il!i~+_-?][}{1)(|\\/tfjrxnuvczXYUJCLQ0OZmwqpdbkhao*#MW&8%B@$";

fn ramp_thresholds(ramp: &str) -> Vec<(f64, char)> {
  let chars: Vec<char> = ramp.chars().collect();
  let n = chars.len();
  chars.iter().enumerate().map(|(i, &c)| {
    let limit = ((i + 1) as f64 / n as f64).min(1.01);
    (limit, c)
  }).collect()
}

pub struct WavePlane;

fn sample_uv(u: f64, v: f64, elapsed: f64, scroll: bool) -> (f64, f64) {
  if !scroll { return (u, v); }
  ((u + elapsed * 0.055).rem_euclid(1.0),
   (v + elapsed * 0.025).rem_euclid(1.0))
}

fn wave_height(u: f64, v: f64, phase: f64, scale: f64) -> f64 {
  let x = 2.0 * PI * u;
  let y = 2.0 * PI * v;
  let t = 2.0 * PI * phase;
  let density = scale.round().max(1.0);
  let mut z = 0.0_f64;
  for &(a, wx, wy, ts, off) in WAVES {
    z += a * ((wx * density) * x + (wy * density) * y + ts * t + off * 2.0 * PI).sin();
  }
  z
}

fn levels(ctx: &FrameContext) -> Vec<f64> {
  let w = ctx.width; let h = ctx.height;
  let mut g = vec![0.0_f64; w * h];
  let contrast = ctx.options.contrast;
  for row in 0..h {
    let v = row as f64 / h as f64;
    let base = row * w;
    for col in 0..w {
      let u = col as f64 / w as f64;
      let (su, sv) = sample_uv(u, v, ctx.elapsed, ctx.options.scroll);
      g[base + col] = clamp(shade(wave_height(su, sv, ctx.phase, ctx.options.scale), contrast));
    }
  }
  g
}

impl Animation for WavePlane {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let g = levels(ctx);
    let th = ramp_thresholds(RAMP_CLEAN);
    render_field(ctx, &g, &th, &STYLE, out);
  }
}
