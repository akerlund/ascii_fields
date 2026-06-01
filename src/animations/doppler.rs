//! Exoplanet Doppler: a star wobbles, wavefronts bunch on the approach side
//! (blueshift) and stretch behind (redshift). Custom truecolor renderer so the
//! shift sign maps to red/blue regardless of theme; falls back to gray when
//! the theme is `grayscale`/`mono`.

use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, density_char, gray_fg, resolve_theme, BLACK_BG, RESET};
use std::fmt::Write;

const TH: &[(f64, char)] = &[
  (0.10, ' '),
  (0.22, '.'),
  (0.34, ':'),
  (0.46, '-'),
  (0.58, '='),
  (0.70, '+'),
  (0.82, '*'),
  (0.92, '#'),
  (1.01, '@'),
];
const C: f64 = 0.42;
const EMIT_DT: f64 = 0.14;

#[derive(Clone, Copy)]
struct Front {
  x: f64,
  y: f64,
  vx: f64,
  vy: f64,
  t0: f64,
}

pub struct Doppler {
  fronts: Vec<Front>,
  next_emit: f64,
  last: f64,
}
impl Default for Doppler {
  fn default() -> Self {
    Self { fronts: Vec::new(), next_emit: 0.0, last: 0.0 }
  }
}

fn shift_color(shift: f64) -> (u8, u8, u8) {
  if shift < 0.5 {
    let t = shift / 0.5;
    (255, (70.0 + 185.0 * t) as u8, (70.0 + 185.0 * t) as u8)
  } else {
    let t = (shift - 0.5) / 0.5;
    ((255.0 - 215.0 * t) as u8, (255.0 - 150.0 * t) as u8, 255)
  }
}

impl Animation for Doppler {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    if ctx.elapsed < self.last {
      self.fronts.clear();
      self.next_emit = 0.0;
    }
    self.last = ctx.elapsed;
    let w = ctx.width;
    let h = ctx.height;
    let ax = w as f64 / (h as f64 * 2.0).max(1.0);
    let omega = 1.4_f64;
    let r_star = 0.18;
    let r_planet = 0.62;
    let sx = r_star * (omega * ctx.elapsed).cos();
    let sy = r_star * (omega * ctx.elapsed).sin();
    let px_pl = -r_planet * (omega * ctx.elapsed).cos();
    let py_pl = -r_planet * (omega * ctx.elapsed).sin();
    while ctx.elapsed >= self.next_emit {
      let tt = self.next_emit;
      let ssx = r_star * (omega * tt).cos();
      let ssy = r_star * (omega * tt).sin();
      let vvx = -r_star * omega * (omega * tt).sin();
      let vvy = r_star * omega * (omega * tt).cos();
      self.fronts.push(Front { x: ssx, y: ssy, vx: vvx, vy: vvy, t0: tt });
      self.next_emit += EMIT_DT;
    }
    self.fronts.retain(|f| ctx.elapsed - f.t0 < 5.0);
    let theme = resolve_theme(&ctx.options.theme, "scene");
    let mono = theme == "mono";
    let bright = ctx.options.brightness;
    let contrast = ctx.options.contrast;
    let dw = (w.saturating_sub(1)).max(1) as f64;
    let dh = (h.saturating_sub(1)).max(1) as f64;
    for row in 0..h {
      out.push_str(BLACK_BG);
      let cy = (row as f64 / dh - 0.5) * 2.0;
      let mut last_color: Option<u8> = None;
      let mut last_rgb: Option<(u8, u8, u8)> = None;
      for col in 0..w {
        let cx = (col as f64 / dw - 0.5) * 2.0 * ax;
        let mut intensity = 0.0_f64;
        let mut dop_acc = 0.0_f64;
        for f in &self.fronts {
          let radius = C * (ctx.elapsed - f.t0);
          let dx = cx - f.x;
          let dy = cy - f.y;
          let dist = (dx * dx + dy * dy).sqrt().max(1e-4);
          let b = (-((dist - radius).powi(2)) / 0.0010).exp();
          if b > 0.001 {
            intensity += b;
            let dop = (f.vx * dx + f.vy * dy) / (dist * C);
            dop_acc += b * dop;
          }
        }
        intensity += (-((cx - sx).powi(2) + (cy - sy).powi(2)) / 0.0016).exp();
        intensity += 0.5 * (-((cx - px_pl).powi(2) + (cy - py_pl).powi(2)) / 0.0010).exp();
        let value = clamp(intensity * contrast);
        let ch = density_char(value, TH);
        if value <= 0.001 {
          out.push(' ');
          last_color = None;
          last_rgb = None;
          continue;
        }
        if mono {
          let color = gray_fg(value, 234, 255, bright);
          if Some(color) != last_color {
            let _ = write!(out, "\x1b[38;5;{}m", color);
            last_color = Some(color);
          }
        } else {
          let shift = clamp(0.5 + 0.5 * (if intensity > 0.0 { dop_acc / intensity } else { 0.0 }) * 1.6);
          let (r, g, b) = shift_color(shift);
          let rgb = ((r as f64 * value) as u8, (g as f64 * value) as u8, (b as f64 * value) as u8);
          if Some(rgb) != last_rgb {
            let _ = write!(out, "\x1b[38;2;{};{};{}m", rgb.0, rgb.1, rgb.2);
            last_rgb = Some(rgb);
          }
        }
        out.push(ch);
      }
      out.push_str(RESET);
      if row + 1 < h {
        out.push('\n');
      }
    }
  }
}
