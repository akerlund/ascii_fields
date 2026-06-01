use std::f64::consts::{PI, TAU};

use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, render_field, FieldStyle};
use crate::noise::fbm_seeded;

use super::field_common::{aspect, dims, pulse, FrameScratch, FIELD_TH, LINE_TH};

const CONVECTION_STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "infrared" };
#[derive(Default)]
pub struct Convection {
  scratch: FrameScratch,
}
impl Animation for Convection {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let (w, h, dw, dh) = dims(ctx);
    let t = ctx.elapsed * 0.55;
    let scale = ctx.options.scale.max(0.4);
    let grid = self.scratch.grid(w * h);
    for row in 0..h {
      let v = row as f64 / dh;
      let y = v * PI;
      let base = row * w;
      for col in 0..w {
        let u = col as f64 / dw;
        let x = (u * 5.0 + 0.25 * (t + v * 3.0).sin()) * scale;
        let rolls = (x * TAU + t).sin() * y.sin();
        let plume = pulse(rolls.abs() - 0.88, 0.004) * (1.0 - v).powf(0.35);
        let cool = pulse(rolls.abs() - 0.10, 0.010) * v.powf(0.7);
        let noise = fbm_seeded(u * 5.0 + t * 0.3, v * 3.0 - t * 0.2, 3, 19);
        let value = 0.38 + 0.28 * rolls + 0.45 * plume - 0.18 * cool + 0.15 * noise;
        grid[base + col] = clamp(0.5 + (value - 0.5) * ctx.options.contrast);
      }
    }
    render_field(ctx, grid, FIELD_TH, &CONVECTION_STYLE, out);
  }
}

const SCHLIEREN_STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "xray" };
#[derive(Default)]
pub struct Schlieren {
  scratch: FrameScratch,
}
impl Animation for Schlieren {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let (w, h, dw, dh) = dims(ctx);
    let ax = aspect(ctx);
    let t = ctx.elapsed;
    let du = 1.0 / dw;
    let dv = 1.0 / dh;
    let sources = [
      (0.28 + 0.08 * (t * 0.7).sin(), 0.55 + 0.12 * (t * 0.5).cos(), 1.0),
      (0.68 + 0.10 * (t * 0.4).cos(), 0.38 + 0.10 * (t * 0.8).sin(), -0.85),
      (0.50 + 0.18 * (t * 0.25).sin(), 0.72, 0.65),
    ];
    let density = |u: f64, v: f64| -> f64 {
      let mut d = 0.0;
      for &(sx, sy, s) in &sources {
        let dx = (u - sx) * ax;
        let dy = v - sy;
        let r2 = dx * dx + dy * dy;
        d += s * (-r2 / 0.010).exp();
        let r = r2.sqrt();
        d += 0.32 * pulse(r - (0.10 + 0.035 * (t * 0.9 + s).sin()), 0.00045);
      }
      d + 0.20 * (u * 16.0 + t * 0.8).sin() * (v * 4.0).sin()
    };
    let grid = self.scratch.grid(w * h);
    for row in 0..h {
      let v = row as f64 / dh;
      let base = row * w;
      for col in 0..w {
        let u = col as f64 / dw;
        let gx = density(u + du, v) - density(u - du, v);
        let gy = density(u, v + dv) - density(u, v - dv);
        let grad = (gx * gx + gy * gy).sqrt() * 8.0;
        grid[base + col] = clamp(grad * ctx.options.contrast);
      }
    }
    render_field(ctx, grid, LINE_TH, &SCHLIEREN_STYLE, out);
  }
}

const FERRO_STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "copper" };
#[derive(Default)]
pub struct Ferrofluid {
  scratch: FrameScratch,
}
impl Animation for Ferrofluid {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let (w, h, dw, dh) = dims(ctx);
    let ax = aspect(ctx);
    let t = ctx.elapsed * 0.8;
    let magnets = [
      (0.40 + 0.10 * t.sin(), 0.48 + 0.08 * (t * 0.7).cos(), 1.0),
      (0.62 + 0.08 * (t * 0.6).cos(), 0.55 + 0.08 * (t * 0.9).sin(), 0.75),
    ];
    let grid = self.scratch.grid(w * h);
    for row in 0..h {
      let v = row as f64 / dh;
      let base = row * w;
      for col in 0..w {
        let u = col as f64 / dw;
        let mut field = 0.0;
        for &(mx, my, strength) in &magnets {
          let dx = (u - mx) * ax;
          let dy = v - my;
          let r = (dx * dx + dy * dy).sqrt().max(0.002);
          let theta = dy.atan2(dx);
          let spikes = (18.0 * theta + t * 2.0 + strength).cos().abs().powf(10.0);
          let ring = pulse(r - (0.07 + 0.012 * (theta * 7.0 + t).sin()), 0.0007);
          field += strength * (0.035 / (r * r + 0.010)).min(1.4) * (0.30 + 0.70 * spikes) + ring * 0.55;
        }
        grid[base + col] = clamp(field * 0.85 * ctx.options.contrast);
      }
    }
    render_field(ctx, grid, LINE_TH, &FERRO_STYLE, out);
  }
}

const RECONNECT_STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "toxic" };
#[derive(Default)]
pub struct Reconnection {
  scratch: FrameScratch,
}
impl Animation for Reconnection {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let (w, h, dw, dh) = dims(ctx);
    let ax = aspect(ctx);
    let t = ctx.elapsed;
    let grid = self.scratch.grid(w * h);
    for row in 0..h {
      let y = row as f64 / dh - 0.5;
      let base = row * w;
      for col in 0..w {
        let x = (col as f64 / dw - 0.5) * ax;
        let sep = 0.16 * (1.0 + 0.25 * (t * 1.2).sin());
        let line1 = pulse(y - (x * 1.15 + sep * x.signum()), 0.0012);
        let line2 = pulse(y + (x * 1.15 + sep * x.signum()), 0.0012);
        let xpoint = pulse(x, 0.0025) * pulse(y, 0.012);
        let jets = pulse(y, 0.0018) * (x.abs() * 7.0 - t * 4.0).sin().abs().powf(8.0);
        grid[base + col] = clamp((line1 + line2 + xpoint + jets * 0.8) * ctx.options.contrast);
      }
    }
    render_field(ctx, grid, LINE_TH, &RECONNECT_STYLE, out);
  }
}
