use std::f64::consts::{PI, TAU};

use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, density_char, render_field, render_glyph_field, FieldStyle};
use crate::noise::{fbm_seeded, star_noise, value_noise};

const FIELD_TH: &[(f64, char)] = &[
  (0.08, ' '), (0.18, '.'), (0.30, ':'), (0.43, '-'), (0.56, '='),
  (0.69, '+'), (0.81, '*'), (0.92, '#'), (1.01, '@'),
];
const LINE_TH: &[(f64, char)] = &[
  (0.08, ' '), (0.20, '.'), (0.36, ':'), (0.52, '-'), (0.68, '='),
  (0.80, '+'), (0.90, '*'), (0.97, '#'), (1.01, '@'),
];

#[inline]
fn dims(ctx: &FrameContext) -> (usize, usize, f64, f64) {
  let w = ctx.width;
  let h = ctx.height;
  let dw = w.saturating_sub(1).max(1) as f64;
  let dh = h.saturating_sub(1).max(1) as f64;
  (w, h, dw, dh)
}

#[inline]
fn aspect(ctx: &FrameContext) -> f64 {
  ctx.width as f64 / (ctx.height as f64 * 2.0).max(1.0)
}

#[inline]
fn pulse(x: f64, width: f64) -> f64 {
  (-(x * x) / width.max(0.00001)).exp()
}

fn put(grid: &mut [f64], glyphs: &mut [char], w: usize, h: usize, col: i64, row: i64, level: f64, ch: char) {
  if col >= 0 && (col as usize) < w && row >= 0 && (row as usize) < h {
    let idx = row as usize * w + col as usize;
    if level > grid[idx] {
      grid[idx] = level;
      glyphs[idx] = ch;
    }
  }
}

fn line(grid: &mut [f64], glyphs: &mut [char], w: usize, h: usize, a: (f64, f64), b: (f64, f64), level: f64, ch: char) {
  let dx = b.0 - a.0;
  let dy = b.1 - a.1;
  let steps = dx.abs().max(dy.abs()).ceil().max(1.0) as i64;
  for i in 0..=steps {
    let f = i as f64 / steps as f64;
    put(grid, glyphs, w, h, (a.0 + dx * f).round() as i64, (a.1 + dy * f).round() as i64, level, ch);
  }
}

const CONVECTION_STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "fire" };
pub struct Convection;
impl Animation for Convection {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let (w, h, dw, dh) = dims(ctx);
    let t = ctx.elapsed * 0.55;
    let scale = ctx.options.scale.max(0.4);
    let mut grid = vec![0.0; w * h];
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
    render_field(ctx, &grid, FIELD_TH, &CONVECTION_STYLE, out);
  }
}

const SCHLIEREN_STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "ice" };
pub struct Schlieren;
impl Animation for Schlieren {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let (w, h, dw, dh) = dims(ctx);
    let ax = aspect(ctx);
    let t = ctx.elapsed;
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
    let mut grid = vec![0.0; w * h];
    for row in 0..h {
      let v = row as f64 / dh;
      let base = row * w;
      for col in 0..w {
        let u = col as f64 / dw;
        let du = 1.0 / dw;
        let dv = 1.0 / dh;
        let gx = density(u + du, v) - density(u - du, v);
        let gy = density(u, v + dv) - density(u, v - dv);
        let grad = (gx * gx + gy * gy).sqrt() * 8.0;
        grid[base + col] = clamp(grad * ctx.options.contrast);
      }
    }
    render_field(ctx, &grid, LINE_TH, &SCHLIEREN_STYLE, out);
  }
}

const FERRO_STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "copper" };
pub struct Ferrofluid;
impl Animation for Ferrofluid {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let (w, h, dw, dh) = dims(ctx);
    let ax = aspect(ctx);
    let t = ctx.elapsed * 0.8;
    let magnets = [
      (0.40 + 0.10 * t.sin(), 0.48 + 0.08 * (t * 0.7).cos(), 1.0),
      (0.62 + 0.08 * (t * 0.6).cos(), 0.55 + 0.08 * (t * 0.9).sin(), 0.75),
    ];
    let mut grid = vec![0.0; w * h];
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
    render_field(ctx, &grid, LINE_TH, &FERRO_STYLE, out);
  }
}

const SEISMO_STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "amber" };
pub struct Seismograph;
impl Animation for Seismograph {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let (w, h, dw, dh) = dims(ctx);
    let ax = aspect(ctx);
    let t = ctx.elapsed;
    let mut grid = vec![0.0; w * h];
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
    render_field(ctx, &grid, LINE_TH, &SEISMO_STYLE, out);
  }
}

const QUASI_STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "spectrum" };
pub struct Quasicrystal;
impl Animation for Quasicrystal {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let (w, h, dw, dh) = dims(ctx);
    let t = ctx.elapsed * 0.35;
    let scale = ctx.options.scale.max(0.35);
    let mut grid = vec![0.0; w * h];
    for row in 0..h {
      let y = (row as f64 / dh - 0.5) * 9.0 * scale;
      let base = row * w;
      for col in 0..w {
        let x = (col as f64 / dw - 0.5) * 9.0 * scale * aspect(ctx);
        let mut s = 0.0;
        for k in 0..5 {
          let a = TAU * k as f64 / 5.0 + 0.12 * t;
          s += (x * a.cos() + y * a.sin() + t * (1.0 + k as f64 * 0.17)).cos();
        }
        let level = 0.5 + 0.5 * (s / 5.0).sin();
        grid[base + col] = clamp(0.5 + (level - 0.5) * ctx.options.contrast * 1.7);
      }
    }
    render_field(ctx, &grid, FIELD_TH, &QUASI_STYLE, out);
  }
}

const MOIRE_STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "mono" };
pub struct Moire;
impl Animation for Moire {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let (w, h, dw, dh) = dims(ctx);
    let t = ctx.elapsed * 0.25;
    let mut grid = vec![0.0; w * h];
    for row in 0..h {
      let y = row as f64 / dh - 0.5;
      let base = row * w;
      for col in 0..w {
        let x = (col as f64 / dw - 0.5) * aspect(ctx);
        let a1 = 0.18 * t.sin();
        let a2 = -0.16 * (t * 0.8).cos();
        let p1 = ((x * a1.cos() + y * a1.sin()) * 90.0 + t * 5.0).sin().abs();
        let p2 = ((x * a2.cos() + y * a2.sin()) * 88.0 - t * 4.0).sin().abs();
        let value = pulse(p1 - 0.05, 0.006) + pulse(p2 - 0.05, 0.006) + 0.35 * (1.0 - (p1 - p2).abs());
        grid[base + col] = clamp(value * ctx.options.contrast);
      }
    }
    render_field(ctx, &grid, LINE_TH, &MOIRE_STYLE, out);
  }
}

const NBODY_STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "nebula" };
pub struct NBody;
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
    let mut grid = vec![0.0; w * h];
    let mut glyphs = vec![' '; w * h];
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
      put(&mut grid, &mut glyphs, w, h, (bx * dw).round() as i64, (by * dh).round() as i64, 1.0, if mass > 0.7 { '@' } else { '*' });
    }
    render_glyph_field(ctx, &grid, &glyphs, &NBODY_STYLE, out);
  }
}

const PULSAR_STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "ice" };
pub struct Pulsar;
impl Animation for Pulsar {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let (w, h, dw, dh) = dims(ctx);
    let ax = aspect(ctx);
    let t = ctx.elapsed;
    let beam = t * 2.8;
    let mut grid = vec![0.0; w * h];
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
    render_field(ctx, &grid, LINE_TH, &PULSAR_STYLE, out);
  }
}

const SUPERNOVA_STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "fire" };
pub struct Supernova;
impl Animation for Supernova {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let (w, h, dw, dh) = dims(ctx);
    let ax = aspect(ctx);
    let age = (ctx.elapsed * 0.22) % 1.0;
    let radius = 0.04 + age * 0.62;
    let mut grid = vec![0.0; w * h];
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
    render_field(ctx, &grid, FIELD_TH, &SUPERNOVA_STYLE, out);
  }
}

const SOLAR_WIND_STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "aurora" };
pub struct SolarWind;
impl Animation for SolarWind {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let (w, h, dw, dh) = dims(ctx);
    let ax = aspect(ctx);
    let t = ctx.elapsed * 0.9;
    let mut grid = vec![0.0; w * h];
    for row in 0..h {
      let v = row as f64 / dh;
      let base = row * w;
      for col in 0..w {
        let u = col as f64 / dw;
        let dx = (u - 0.63) * ax;
        let dy = v - 0.5;
        let r = (dx * dx + dy * dy).sqrt();
        let bow = pulse(r - 0.17, 0.0006) * if u < 0.63 { 1.0 } else { 0.25 };
        let stream = ((v * 28.0 + 2.0 * (u * 5.0 + t).sin()).sin().abs()).powf(12.0) * (1.0 - u).max(0.0);
        let wake = pulse(dy + 0.10 * (u * 9.0 - t).sin(), 0.002) * if u > 0.63 { 0.7 } else { 0.0 };
        grid[base + col] = clamp((0.45 * stream + bow + wake) * ctx.options.contrast);
      }
    }
    render_field(ctx, &grid, LINE_TH, &SOLAR_WIND_STYLE, out);
  }
}

const RECONNECT_STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "plasma" };
pub struct Reconnection;
impl Animation for Reconnection {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let (w, h, dw, dh) = dims(ctx);
    let t = ctx.elapsed;
    let mut grid = vec![0.0; w * h];
    for row in 0..h {
      let y = row as f64 / dh - 0.5;
      let base = row * w;
      for col in 0..w {
        let x = (col as f64 / dw - 0.5) * aspect(ctx);
        let sep = 0.16 * (1.0 + 0.25 * (t * 1.2).sin());
        let line1 = pulse(y - (x * 1.15 + sep * x.signum()), 0.0012);
        let line2 = pulse(y + (x * 1.15 + sep * x.signum()), 0.0012);
        let xpoint = pulse(x, 0.0025) * pulse(y, 0.012);
        let jets = pulse(y, 0.0018) * (x.abs() * 7.0 - t * 4.0).sin().abs().powf(8.0);
        grid[base + col] = clamp((line1 + line2 + xpoint + jets * 0.8) * ctx.options.contrast);
      }
    }
    render_field(ctx, &grid, LINE_TH, &RECONNECT_STYLE, out);
  }
}

const CAUSTICS_STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "ocean" };
pub struct Caustics;
impl Animation for Caustics {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let (w, h, dw, dh) = dims(ctx);
    let t = ctx.elapsed * 0.55;
    let mut grid = vec![0.0; w * h];
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
    render_field(ctx, &grid, LINE_TH, &CAUSTICS_STYLE, out);
  }
}

const DUNES_STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "amber" };
pub struct Dunes;
impl Animation for Dunes {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let (w, h, dw, dh) = dims(ctx);
    let t = ctx.elapsed * 0.18;
    let mut grid = vec![0.0; w * h];
    for row in 0..h {
      let v = row as f64 / dh;
      let base = row * w;
      for col in 0..w {
        let u = col as f64 / dw;
        let slope = v + 0.08 * (u * 5.0 + t).sin();
        let ripples = ((u * 34.0 + slope * 14.0 - t * 5.0 + fbm_seeded(u * 4.0, v * 4.0, 3, 23) * 3.0).sin() * 0.5 + 0.5).powf(3.0);
        let ridge = pulse(ripples - 0.92, 0.010);
        let shade = 0.28 + 0.45 * v + 0.35 * ridge;
        grid[base + col] = clamp(shade * ctx.options.contrast);
      }
    }
    render_field(ctx, &grid, FIELD_TH, &DUNES_STYLE, out);
  }
}

const TOPO_STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "ice" };
pub struct Topography;
impl Animation for Topography {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let (w, h, dw, dh) = dims(ctx);
    let t = ctx.elapsed * 0.12;
    let mut grid = vec![0.0; w * h];
    for row in 0..h {
      let v = row as f64 / dh;
      let base = row * w;
      for col in 0..w {
        let u = col as f64 / dw;
        let elev = fbm_seeded(u * 4.2 + t, v * 4.2 - t * 0.5, 5, 771);
        let contour = pulse(((elev * 15.0).fract() - 0.5).abs(), 0.0025);
        let river = pulse(elev - (0.43 + 0.04 * (u * 8.0 + t * 4.0).sin()), 0.0018) * (0.6 + 0.4 * (v * 10.0).sin().abs());
        grid[base + col] = clamp((contour * 0.85 + river * 0.75 + elev * 0.20) * ctx.options.contrast);
      }
    }
    render_field(ctx, &grid, LINE_TH, &TOPO_STYLE, out);
  }
}

const SCOPE_STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "aurora" };
pub struct Oscilloscope;
impl Animation for Oscilloscope {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let (w, h, dw, dh) = dims(ctx);
    let t = ctx.elapsed;
    let mut grid = vec![0.0; w * h];
    let mut glyphs = vec![' '; w * h];
    for row in (0..h).step_by(4.max(h / 8)) {
      for col in 0..w {
        put(&mut grid, &mut glyphs, w, h, col as i64, row as i64, 0.16, '.');
      }
    }
    for col in (0..w).step_by(8.max(w / 12)) {
      for row in 0..h {
        put(&mut grid, &mut glyphs, w, h, col as i64, row as i64, 0.12, '.');
      }
    }
    let mid = dh * 0.48;
    let amp = dh * 0.28 * ctx.options.scale.min(2.0);
    let mut prev: Option<(f64, f64)> = None;
    for col in 0..w {
      let u = col as f64 / dw;
      let y = mid + amp * ((u * TAU * 3.0 + t * 2.1).sin() * 0.65 + (u * TAU * 7.0 - t * 1.2).sin() * 0.25);
      if let Some(p) = prev {
        line(&mut grid, &mut glyphs, w, h, p, (col as f64, y), 0.92, '*');
      }
      prev = Some((col as f64, y));
    }
    render_glyph_field(ctx, &grid, &glyphs, &SCOPE_STYLE, out);
  }
}

const RADAR_STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "aurora" };
pub struct Radar;
impl Animation for Radar {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let (w, h, dw, dh) = dims(ctx);
    let ax = aspect(ctx);
    let sweep = ctx.elapsed * 1.4;
    let contacts = [(0.30, 0.32), (0.74, 0.42), (0.58, 0.76), (0.42, 0.62)];
    let mut grid = vec![0.0; w * h];
    for row in 0..h {
      let v = row as f64 / dh;
      let base = row * w;
      for col in 0..w {
        let u = col as f64 / dw;
        let dx = (u - 0.5) * ax;
        let dy = v - 0.5;
        let r = (dx * dx + dy * dy).sqrt();
        let a = dy.atan2(dx);
        let rings = pulse((r * 8.0).fract() - 0.5, 0.002) * 0.22;
        let da = (a - sweep + PI).rem_euclid(TAU) - PI;
        let sweep_line = pulse(da, 0.006) * (1.0 - r * 1.4).max(0.0);
        let mut blip = 0.0;
        for &(cx, cy) in &contacts {
          let bx = (u - cx) * ax;
          let by = v - cy;
          let br = (bx * bx + by * by).sqrt();
          let ba = by.atan2(bx);
          let lit = 0.25 + 0.75 * pulse((ba - sweep + PI).rem_euclid(TAU) - PI, 0.080);
          blip += pulse(br, 0.0007) * lit;
        }
        grid[base + col] = clamp((rings + sweep_line + blip) * ctx.options.contrast);
      }
    }
    render_field(ctx, &grid, LINE_TH, &RADAR_STYLE, out);
  }
}

const PENROSE_STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "rose" };
pub struct Penrose;
impl Animation for Penrose {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let (w, h, dw, dh) = dims(ctx);
    let t = ctx.elapsed * 0.10;
    let phi = 1.61803398875;
    let mut grid = vec![0.0; w * h];
    for row in 0..h {
      let y = (row as f64 / dh - 0.5) * 8.0;
      let base = row * w;
      for col in 0..w {
        let x = (col as f64 / dw - 0.5) * 8.0 * aspect(ctx);
        let mut edge = 0.0;
        for k in 0..5 {
          let a = TAU * k as f64 / 5.0;
          let p = x * a.cos() + y * a.sin() + t;
          let q = (p * phi).fract();
          edge += pulse((q - 0.5).abs(), 0.002);
        }
        grid[base + col] = clamp(edge * 0.55 * ctx.options.contrast);
      }
    }
    render_field(ctx, &grid, LINE_TH, &PENROSE_STYLE, out);
  }
}

const VORONOI_STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "copper" };
pub struct Voronoi;
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
    let mut grid = vec![0.0; w * h];
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
          if d < d1 { d2 = d1; d1 = d; } else if d < d2 { d2 = d; }
        }
        let edge = pulse((d2.sqrt() - d1.sqrt()).abs(), 0.00035);
        grid[base + col] = clamp(edge * ctx.options.contrast);
      }
    }
    render_field(ctx, &grid, LINE_TH, &VORONOI_STYLE, out);
  }
}

const REACTION_STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "plasma" };
pub struct ReactionRings;
impl Animation for ReactionRings {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let (w, h, dw, dh) = dims(ctx);
    let ax = aspect(ctx);
    let t = ctx.elapsed * 0.55;
    let centers = [(0.28, 0.36, 0.0), (0.68, 0.44, 1.7), (0.48, 0.72, 3.1), (0.78, 0.76, 4.6)];
    let mut grid = vec![0.0; w * h];
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
    render_field(ctx, &grid, FIELD_TH, &REACTION_STYLE, out);
  }
}

const STRANGE_STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "spectrum" };
pub struct Strange;
impl Animation for Strange {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let w = ctx.width;
    let h = ctx.height;
    let mut grid = vec![0.0; w * h];
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
      let col = ((x + 2.0) / 4.0 * (w.saturating_sub(1) as f64)).round() as i64;
      let row = ((y + 2.0) / 4.0 * (h.saturating_sub(1) as f64)).round() as i64;
      if col >= 0 && (col as usize) < w && row >= 0 && (row as usize) < h {
        let idx = row as usize * w + col as usize;
        grid[idx] = (grid[idx] + 0.16_f64).min(1.0_f64);
      }
    }
    for v in &mut grid { *v = clamp(*v * ctx.options.contrast); }
    render_field(ctx, &grid, LINE_TH, &STRANGE_STYLE, out);
  }
}
