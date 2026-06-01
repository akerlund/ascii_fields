use std::f64::consts::{PI, TAU};

use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, density_char, render_field, render_glyph_field, FieldStyle};
use crate::noise::{fbm_seeded, star_noise};

use super::field_common::{aspect, dims, pulse, put, FrameScratch, FIELD_TH, LINE_TH};

const NBODY_STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "nebula" };
const STELLAR_STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "stellar" };

#[derive(Default)]
pub struct NBody { scratch: FrameScratch }
impl Animation for NBody {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let (w, h, dw, dh) = dims(ctx);
    let ax = aspect(ctx);
    let t = ctx.elapsed * 0.33;
    let bodies = [
      (0.50 + 0.23 * t.cos(), 0.50 + 0.20 * t.sin(), 1.0),
      (0.50 + 0.16 * (t * 1.7 + 2.1).cos(), 0.50 + 0.26 * (t * 1.7 + 2.1).sin(), 0.72),
      (0.50 + 0.32 * (t * 0.73 + 4.0).cos(), 0.50 + 0.10 * (t * 0.73 + 4.0).sin(), 0.55),
      (0.50 + 0.10 * (t * 2.3 + 1.0).cos(), 0.50 + 0.34 * (t * 2.3 + 1.0).sin(), 0.35),
    ];
    let (grid, glyphs) = self.scratch.grid_and_glyphs(w * h);
    for row in 0..h {
      let v = row as f64 / dh;
      for col in 0..w {
        let u = col as f64 / dw;
        let mut value = 0.0;
        for &(bx, by, mass) in &bodies {
          let dx = (u - bx) * ax;
          let dy = v - by;
          value += mass * 0.018 / (dx * dx + dy * dy + 0.002);
        }
        grid[row * w + col] = clamp(value * ctx.options.contrast);
      }
    }
    for (idx, glyph) in glyphs.iter_mut().enumerate() {
      *glyph = density_char(grid[idx], FIELD_TH);
    }
    for &(bx, by, mass) in &bodies {
      put(grid, glyphs, w, h, (bx * dw).round() as i64, (by * dh).round() as i64, 1.0, if mass > 0.7 { '@' } else { '*' });
    }
    render_glyph_field(ctx, grid, glyphs, &NBODY_STYLE, out);
  }
}

#[derive(Default)]
pub struct Pulsar { scratch: FrameScratch }
impl Animation for Pulsar {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let (w, h, dw, dh) = dims(ctx);
    let ax = aspect(ctx);
    let t = ctx.elapsed;
    let beam = t * 2.8;
    let grid = self.scratch.grid(w * h);
    for row in 0..h {
      let v = row as f64 / dh;
      let base = row * w;
      for col in 0..w {
        let u = col as f64 / dw;
        let dx = (u - 0.5) * ax;
        let dy = v - 0.5;
        let r = (dx * dx + dy * dy).sqrt();
        let a = dy.atan2(dx);
        let da = (a - beam + PI).rem_euclid(TAU) - PI;
        let beam_level = pulse(da, 0.020) * (1.0 - r * 1.6).max(0.0);
        let star = if star_noise(col as i64, row as i64) > 0.997 { 0.8 } else { 0.0 };
        let core = pulse(r, 0.0018);
        grid[base + col] = clamp((beam_level + star + core) * ctx.options.contrast);
      }
    }
    render_field(ctx, grid, LINE_TH, &STELLAR_STYLE, out);
  }
}

#[derive(Default)]
pub struct Supernova { scratch: FrameScratch }
impl Animation for Supernova {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let (w, h, dw, dh) = dims(ctx);
    let ax = aspect(ctx);
    let age = (ctx.elapsed * 0.22) % 1.0;
    let radius = 0.04 + age * 0.62;
    let grid = self.scratch.grid(w * h);
    for row in 0..h {
      let v = row as f64 / dh;
      let base = row * w;
      for col in 0..w {
        let u = col as f64 / dw;
        let dx = (u - 0.5) * ax;
        let dy = v - 0.5;
        let r = (dx * dx + dy * dy).sqrt();
        let a = dy.atan2(dx);
        let filament = 0.7 + 0.3 * (a * 13.0 + fbm_seeded(u * 5.0, v * 5.0, 3, 88) * 6.0).sin().abs();
        let shell = pulse(r - radius * filament, 0.0012 + age * 0.003) * (1.0 - age).powf(0.35);
        let core = pulse(r, 0.008 + age * 0.050) * (1.0 - age);
        grid[base + col] = clamp((shell + core) * ctx.options.contrast);
      }
    }
    render_field(ctx, grid, FIELD_TH, &STELLAR_STYLE, out);
  }
}

const SOLAR_WIND_STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "aurora" };
#[derive(Default)]
pub struct SolarWind { scratch: FrameScratch }
impl Animation for SolarWind {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let (w, h, dw, dh) = dims(ctx);
    let ax = aspect(ctx);
    let sun_x = 0.63;
    let t = ctx.elapsed * 0.9;
    let grid = self.scratch.grid(w * h);
    for row in 0..h {
      let v = row as f64 / dh;
      let base = row * w;
      for col in 0..w {
        let u = col as f64 / dw;
        let dx = (u - sun_x) * ax;
        let dy = v - 0.5;
        let r = (dx * dx + dy * dy).sqrt();
        let bow = pulse(r - 0.17, 0.0006) * if u < sun_x { 1.0 } else { 0.25 };
        let stream = ((v * 28.0 + 2.0 * (u * 5.0 + t).sin()).sin().abs()).powf(12.0) * (1.0 - u).max(0.0);
        let wake = pulse(dy + 0.10 * (u * 9.0 - t).sin(), 0.002) * if u > sun_x { 0.7 } else { 0.0 };
        grid[base + col] = clamp((0.45 * stream + bow + wake) * ctx.options.contrast);
      }
    }
    render_field(ctx, grid, LINE_TH, &SOLAR_WIND_STYLE, out);
  }
}
