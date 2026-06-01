use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, render_field, FieldStyle};
use crate::noise::fbm_seeded;

use super::field_common::{aspect, dims, pulse, FrameScratch, FIELD_TH, LINE_TH};

const SEISMO_STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "geologic" };
#[derive(Default)]
pub struct Seismograph {
  scratch: FrameScratch,
}
impl Animation for Seismograph {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let (w, h, dw, dh) = dims(ctx);
    let ax = aspect(ctx);
    let t = ctx.elapsed;
    let grid = self.scratch.grid(w * h);
    let epicenters = [(0.22, 0.72, 0.0), (0.70, 0.58, 2.4)];
    for row in 0..h {
      let v = row as f64 / dh;
      let layer = 0.10 * (v * 24.0 + 0.8 * (t + v).sin()).sin().abs();
      let base = row * w;
      for col in 0..w {
        let u = col as f64 / dw;
        let mut value = layer;
        for &(ex, ey, off) in &epicenters {
          let age = (t + off) % 7.0;
          let dx = (u - ex) * ax;
          let dy = v - ey;
          let r = (dx * dx + dy * dy).sqrt();
          value += pulse(r - age * 0.085, 0.0009) * (1.0 - age / 7.0).max(0.0);
          value += 0.55 * pulse(r - age * 0.052, 0.00055) * (1.0 - age / 7.0).max(0.0);
        }
        if (v - 0.78 - 0.03 * (u * 10.0).sin()).abs() < 0.008 {
          value += 0.35;
        }
        grid[base + col] = clamp(value * ctx.options.contrast);
      }
    }
    render_field(ctx, grid, LINE_TH, &SEISMO_STYLE, out);
  }
}

const CAUSTICS_STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "bathymetry" };
#[derive(Default)]
pub struct Caustics {
  scratch: FrameScratch,
}
impl Animation for Caustics {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let (w, h, dw, dh) = dims(ctx);
    let t = ctx.elapsed * 0.55;
    let grid = self.scratch.grid(w * h);
    for row in 0..h {
      let v = row as f64 / dh;
      let base = row * w;
      for col in 0..w {
        let u = col as f64 / dw;
        let a = (u * 14.0 + (v * 8.0 + t).sin() * 1.6 + t).sin();
        let b = (v * 12.0 + (u * 10.0 - t * 0.8).cos() * 1.4 - t * 0.7).sin();
        let c = ((u + v) * 10.0 + t * 0.5).sin();
        let value = pulse(a, 0.018) + pulse(b, 0.018) + 0.6 * pulse(c, 0.014);
        grid[base + col] = clamp(value * 0.65 * ctx.options.contrast);
      }
    }
    render_field(ctx, grid, LINE_TH, &CAUSTICS_STYLE, out);
  }
}

const DUNES_STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "dusk" };
#[derive(Default)]
pub struct Dunes {
  scratch: FrameScratch,
}
impl Animation for Dunes {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let (w, h, dw, dh) = dims(ctx);
    let t = ctx.elapsed * 0.18;
    let grid = self.scratch.grid(w * h);
    for row in 0..h {
      let v = row as f64 / dh;
      let base = row * w;
      for col in 0..w {
        let u = col as f64 / dw;
        let slope = v + 0.08 * (u * 5.0 + t).sin();
        let ripples =
          ((u * 34.0 + slope * 14.0 - t * 5.0 + fbm_seeded(u * 4.0, v * 4.0, 3, 23) * 3.0).sin() * 0.5 + 0.5)
            .powf(3.0);
        let ridge = pulse(ripples - 0.92, 0.010);
        let shade = 0.28 + 0.45 * v + 0.35 * ridge;
        grid[base + col] = clamp(shade * ctx.options.contrast);
      }
    }
    render_field(ctx, grid, FIELD_TH, &DUNES_STYLE, out);
  }
}

const TOPO_STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "geologic" };
#[derive(Default)]
pub struct Topography {
  scratch: FrameScratch,
}
impl Animation for Topography {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let (w, h, dw, dh) = dims(ctx);
    let t = ctx.elapsed * 0.12;
    let grid = self.scratch.grid(w * h);
    for row in 0..h {
      let v = row as f64 / dh;
      let base = row * w;
      for col in 0..w {
        let u = col as f64 / dw;
        let elev = fbm_seeded(u * 4.2 + t, v * 4.2 - t * 0.5, 5, 771);
        let contour = pulse(((elev * 15.0).fract() - 0.5).abs(), 0.0025);
        let river = pulse(elev - (0.43 + 0.04 * (u * 8.0 + t * 4.0).sin()), 0.0018)
          * (0.6 + 0.4 * (v * 10.0).sin().abs());
        grid[base + col] = clamp((contour * 0.85 + river * 0.75 + elev * 0.20) * ctx.options.contrast);
      }
    }
    render_field(ctx, grid, LINE_TH, &TOPO_STYLE, out);
  }
}
