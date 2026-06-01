use rayon::prelude::*;

use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, render_field, FieldStyle};
use crate::noise::fbm;

const STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "lava" };
const TH: &[(f64, char)] = &[
  (0.07, ' '),
  (0.16, '.'),
  (0.27, ':'),
  (0.39, '-'),
  (0.52, '='),
  (0.65, '+'),
  (0.78, '*'),
  (0.90, '#'),
  (1.01, '@'),
];
// (bx, by, period, offset, radius)
const BUBBLES: &[(f64, f64, f64, f64, f64)] = &[
  (0.18, 0.82, 5.0, 0.00, 0.08),
  (0.34, 0.76, 6.4, 0.23, 0.06),
  (0.56, 0.84, 4.6, 0.47, 0.09),
  (0.74, 0.78, 7.2, 0.68, 0.055),
  (0.88, 0.86, 5.7, 0.81, 0.07),
];

struct BubbleState {
  bx_eff: f64,
  rise: f64,
  inv_rx: f64, // 1 / radius, hoisted out of the per-cell loop
  inv_ry: f64, // 1 / (radius * 0.55), same
  local: f64,
  bursting: bool,
  ring_r: f64,
  ring_decay: f64,
  spark_x_scale: f64, // 140.0 (per-cell coefficient on u)
  spark_phase: f64,   // 140 * 0.01 * offset, pre-shifted
}

pub struct Lava;

impl Animation for Lava {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let w = ctx.width;
    let h = ctx.height;
    let contrast = ctx.options.contrast;
    let bubble_state: Vec<BubbleState> = BUBBLES
      .iter()
      .map(|&(bx, by, period, offset, radius)| {
        let local = ((ctx.elapsed / period + offset) % 1.0 + 1.0) % 1.0;
        let rise = by - local * 0.42;
        let wobble = 0.035 * (ctx.elapsed * 1.3 + offset * 9.0).sin();
        let bx_eff = bx + wobble;
        let bursting = local > 0.78;
        let burst = if bursting { (local - 0.78) / 0.22 } else { 0.0 };
        let ring_r = if bursting { 0.45 + burst * 2.1 } else { 0.0 };
        let ring_decay = if bursting { (1.0 - burst).powf(1.5) } else { 0.0 };
        let inv_rx = 1.0 / radius.max(0.001);
        let inv_ry = inv_rx / 0.55;
        BubbleState {
          bx_eff,
          rise,
          inv_rx,
          inv_ry,
          local,
          bursting,
          ring_r,
          ring_decay,
          spark_x_scale: 140.0,
          spark_phase: offset * 140.0 * 0.01,
        }
      })
      .collect();

    // Frame-invariant terms hoisted out of the row/col loops.
    let drift = ctx.elapsed * 0.18;
    let elapsed_conv = ctx.elapsed * 0.10;
    let elapsed_crack = ctx.elapsed * 0.8;
    let elapsed_spark = ctx.elapsed * 9.0;
    let mut grid = vec![0.0_f64; w * h];
    let dw = (w.saturating_sub(1)).max(1) as f64;
    let dh = (h.saturating_sub(1)).max(1) as f64;
    let bubble_state = &bubble_state; // shared, captured by ref into closures
    grid.par_chunks_mut(w).enumerate().for_each(|(row, row_slice)| {
      let v = row as f64 / dh;
      let one_minus_v = 1.0 - v;
      let sin_v7 = (v * 7.0).sin() * 1.6;
      let v_drift = v * 2.4 - drift; // fbm Y argument is row-only
      let base_heat = 0.16 + 0.22 * one_minus_v; // row-only part of `heat`
      for col in 0..w {
        let u = col as f64 / dw;
        let convection = fbm(u * 3.0 + elapsed_conv, v_drift, 4);
        let cracks = (u * 18.0 + sin_v7 - elapsed_crack).sin().abs();
        let mut heat = base_heat + 0.52 * convection;
        heat -= (cracks - 0.55).max(0.0) * 0.40;
        for b in bubble_state {
          let dx = (u - b.bx_eff) * b.inv_rx;
          let dy = (v - b.rise) * b.inv_ry;
          let dist2 = dx * dx + dy * dy;
          let bubble = (-dist2 * 1.7).exp();
          heat += bubble * (0.34 + 0.22 * b.local);
          if b.bursting {
            let dist = dist2.sqrt();
            let ring = (-((dist - b.ring_r).powi(2)) / 0.08).exp() * b.ring_decay;
            let sparks = ((u + b.spark_phase) * b.spark_x_scale + elapsed_spark).sin().max(0.0);
            heat += ring * (0.60 + 0.30 * sparks);
          }
        }
        row_slice[col] = clamp(heat * contrast);
      }
    });
    render_field(ctx, &grid, TH, &STYLE, out);
  }
}
