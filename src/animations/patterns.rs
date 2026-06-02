use std::f64::consts::TAU;

use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, render_field, FieldStyle};
use crate::noise::value_noise;

use super::field_common::{aspect, dims, pulse, FrameScratch, FIELD_TH, LINE_TH};

const PHI: f64 = 1.618_033_988_749_895;

const QUASI_STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "toxic" };
#[derive(Default)]
pub struct Quasicrystal {
  scratch: FrameScratch,
}
impl Animation for Quasicrystal {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let (w, h, dw, dh) = dims(ctx);
    let ax = aspect(ctx);
    let t = ctx.elapsed * 0.35;
    let scale = ctx.options.scale.max(0.35);
    let grid = self.scratch.grid(w * h);
    for row in 0..h {
      let y = (row as f64 / dh - 0.5) * 9.0 * scale;
      let base = row * w;
      for col in 0..w {
        let x = (col as f64 / dw - 0.5) * 9.0 * scale * ax;
        let mut s = 0.0;
        for k in 0..5 {
          let a = TAU * k as f64 / 5.0 + 0.12 * t;
          s += (x * a.cos() + y * a.sin() + t * (1.0 + k as f64 * 0.17)).cos();
        }
        let level = 0.5 + 0.5 * (s / 5.0).sin();
        grid[base + col] = clamp(0.5 + (level - 0.5) * ctx.options.contrast * 1.7);
      }
    }
    render_field(ctx, grid, FIELD_TH, &QUASI_STYLE, out);
  }
}

const MOIRE_STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "xray" };
#[derive(Default)]
pub struct Moire {
  scratch: FrameScratch,
}
impl Animation for Moire {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let (w, h, dw, dh) = dims(ctx);
    let ax = aspect(ctx);
    let t = ctx.elapsed * 0.25;
    let grid = self.scratch.grid(w * h);
    for row in 0..h {
      let y = row as f64 / dh - 0.5;
      let base = row * w;
      for col in 0..w {
        let x = (col as f64 / dw - 0.5) * ax;
        let a1 = 0.18 * t.sin();
        let a2 = -0.16 * (t * 0.8).cos();
        let p1 = ((x * a1.cos() + y * a1.sin()) * 90.0 + t * 5.0).sin().abs();
        let p2 = ((x * a2.cos() + y * a2.sin()) * 88.0 - t * 4.0).sin().abs();
        let value = pulse(p1 - 0.05, 0.006) + pulse(p2 - 0.05, 0.006) + 0.35 * (1.0 - (p1 - p2).abs());
        grid[base + col] = clamp(value * ctx.options.contrast);
      }
    }
    render_field(ctx, grid, LINE_TH, &MOIRE_STYLE, out);
  }
}

const PENROSE_STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "toxic" };
#[derive(Default)]
pub struct Penrose {
  scratch: FrameScratch,
}
impl Animation for Penrose {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let (w, h, dw, dh) = dims(ctx);
    let ax = aspect(ctx);
    let t = ctx.elapsed * 0.10;
    let grid = self.scratch.grid(w * h);
    for row in 0..h {
      let y = (row as f64 / dh - 0.5) * 8.0;
      let base = row * w;
      for col in 0..w {
        let x = (col as f64 / dw - 0.5) * 8.0 * ax;
        let mut edge = 0.0;
        for k in 0..5 {
          let a = TAU * k as f64 / 5.0;
          let p = x * a.cos() + y * a.sin() + t;
          let q = (p * PHI).fract();
          edge += pulse((q - 0.5).abs(), 0.002);
        }
        grid[base + col] = clamp(edge * 0.55 * ctx.options.contrast);
      }
    }
    render_field(ctx, grid, LINE_TH, &PENROSE_STYLE, out);
  }
}

const VORONOI_STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "geologic" };
#[derive(Default)]
pub struct Voronoi {
  scratch: FrameScratch,
}
impl Animation for Voronoi {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let (w, h, dw, dh) = dims(ctx);
    let ax = aspect(ctx);
    let t = ctx.elapsed * 0.35;
    let mut sites = [(0.0, 0.0); 9];
    for (i, site) in sites.iter_mut().enumerate() {
      let a = i as f64 * 2.399963 + t * (0.7 + i as f64 * 0.03);
      let r = 0.12 + 0.34 * value_noise(i as f64, 2.0, 91);
      *site = (0.5 + r * a.cos(), 0.5 + r * a.sin());
    }
    let grid = self.scratch.grid(w * h);
    for row in 0..h {
      let v = row as f64 / dh;
      let base = row * w;
      for col in 0..w {
        let u = col as f64 / dw;
        let mut d1 = f64::INFINITY;
        let mut d2 = f64::INFINITY;
        for &(sx, sy) in &sites {
          let dx = (u - sx) * ax;
          let dy = v - sy;
          let d = dx * dx + dy * dy;
          if d < d1 {
            d2 = d1;
            d1 = d;
          } else if d < d2 {
            d2 = d;
          }
        }
        let edge = pulse((d2.sqrt() - d1.sqrt()).abs(), 0.00035);
        grid[base + col] = clamp(edge * ctx.options.contrast);
      }
    }
    render_field(ctx, grid, LINE_TH, &VORONOI_STYLE, out);
  }
}

const REACTION_STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "toxic" };
#[derive(Default)]
pub struct ReactionRings {
  scratch: FrameScratch,
}
impl Animation for ReactionRings {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let (w, h, dw, dh) = dims(ctx);
    let ax = aspect(ctx);
    let t = ctx.elapsed * 0.55;
    let centers = [(0.28, 0.36, 0.0), (0.68, 0.44, 1.7), (0.48, 0.72, 3.1), (0.78, 0.76, 4.6)];
    let grid = self.scratch.grid(w * h);
    for row in 0..h {
      let v = row as f64 / dh;
      let base = row * w;
      for col in 0..w {
        let u = col as f64 / dw;
        let mut value = 0.0;
        for &(cx, cy, off) in &centers {
          let dx = (u - cx) * ax;
          let dy = v - cy;
          let r = (dx * dx + dy * dy).sqrt();
          let phase = (r * 34.0 - t * 6.0 - off).sin();
          value += pulse(phase, 0.035) * (1.0 - r * 1.3).max(0.0);
        }
        grid[base + col] = clamp(value * 0.70 * ctx.options.contrast);
      }
    }
    render_field(ctx, grid, FIELD_TH, &REACTION_STYLE, out);
  }
}

const STRANGE_STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "spectrum" };
#[derive(Default)]
pub struct Strange {
  scratch: FrameScratch,
}
impl Animation for Strange {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let (w, h, dw, dh) = dims(ctx);
    let grid = self.scratch.grid(w * h);
    let t = ctx.elapsed * 0.18;
    let a = -1.7 + 0.25 * t.sin();
    let b = 1.8 + 0.20 * (t * 0.7).cos();
    let c = -0.9 + 0.22 * (t * 1.1).sin();
    let d = -1.4 + 0.18 * (t * 0.9).cos();
    let mut x = 0.1;
    let mut y = 0.0;
    let samples = (7000.0 * ctx.options.scale.max(0.5)) as usize;
    for _ in 0..samples {
      let nx = (a * y).sin() - (b * x).cos();
      let ny = (c * x).sin() - (d * y).cos();
      x = nx;
      y = ny;
      let col = ((x + 2.0) / 4.0 * dw).round() as i64;
      let row = ((y + 2.0) / 4.0 * dh).round() as i64;
      if col >= 0 && (col as usize) < w && row >= 0 && (row as usize) < h {
        let idx = row as usize * w + col as usize;
        grid[idx] = (grid[idx] + 0.16).min(1.0);
      }
    }
    for v in grid.iter_mut() {
      *v = clamp(*v * ctx.options.contrast);
    }
    render_field(ctx, grid, LINE_TH, &STRANGE_STYLE, out);
  }
}
