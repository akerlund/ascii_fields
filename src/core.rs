//! Field renderers and shared math helpers. Three renderers:
//!
//! * [`render_field`] – the workhorse, glyph chosen from a level/threshold
//!   ramp, foreground colour from a 256-color gray ramp or a truecolor theme.
//! * [`render_glyph_field`] – explicit per-cell glyph, level only used for
//!   the colour.
//! * [`render_block_field`] – solid background blocks (wave-plane).

use std::fmt::Write;

use crate::animation::FrameContext;
use crate::themes::palette_sample;

pub const BLACK_BG: &str = "\x1b[48;5;232m";
pub const RESET: &str = "\x1b[0m";

#[inline] pub fn clamp(v: f64) -> f64 { v.clamp(0.0, 1.0) }

#[inline]
pub fn smoothstep(edge0: f64, edge1: f64, value: f64) -> f64 {
  let t = clamp((value - edge0) / (edge1 - edge0));
  t * t * (3.0 - 2.0 * t)
}

#[inline]
pub fn shade(value: f64, contrast: f64) -> f64 {
  clamp(0.5 + value * 0.42 * contrast)
}

pub static DEFAULT_THRESHOLDS: &[(f64, char)] = &[
  (0.12, ' '), (0.20, '.'), (0.30, ':'), (0.41, '-'), (0.53, '='),
  (0.66, '+'), (0.79, '*'), (0.91, '#'), (1.01, '%'),
];

#[inline]
pub fn density_char(level: f64, thresholds: &[(f64, char)]) -> char {
  for &(limit, ch) in thresholds {
    if level < limit { return ch; }
  }
  thresholds.last().map(|&(_, ch)| ch).unwrap_or(' ')
}

#[inline]
pub fn gray_fg(level: f64, lo: u8, hi: u8, brightness: f64) -> u8 {
  let v = clamp(level * brightness);
  lo + ((hi - lo) as f64 * v).round() as u8
}

#[inline]
pub fn resolve_theme<'a>(opt: &'a str, default_theme: &'a str) -> &'a str {
  match opt {
    "grayscale" => "mono",
    "scene" => default_theme,
    other => other,
  }
}

pub struct FieldStyle {
  pub gray_lo: u8,
  pub gray_hi: u8,
  pub default_theme: &'static str,
}

impl Default for FieldStyle {
  fn default() -> Self { Self { gray_lo: 234, gray_hi: 255, default_theme: "mono" } }
}

fn write_color_escape(out: &mut String, color: u8) {
  let _ = write!(out, "\x1b[38;5;{}m", color);
}
fn write_rgb_escape(out: &mut String, (r, g, b): (u8, u8, u8)) {
  let _ = write!(out, "\x1b[38;2;{};{};{}m", r, g, b);
}

/// Render a level grid; glyph comes from the thresholds.
pub fn render_field(
  ctx: &FrameContext,
  grid: &[f64],
  thresholds: &[(f64, char)],
  style: &FieldStyle,
  out: &mut String,
) {
  let theme = resolve_theme(&ctx.options.theme, style.default_theme);
  let bright = ctx.options.brightness;
  let w = ctx.width;
  let h = ctx.height;
  let is_mono = theme == "mono";

  for row in 0..h {
    out.push_str(BLACK_BG);
    let base = row * w;
    if is_mono {
      let mut last: Option<u8> = None;
      for col in 0..w {
        let level = grid[base + col];
        let color = gray_fg(level, style.gray_lo, style.gray_hi, bright);
        let ch = density_char(level, thresholds);
        if Some(color) != last { write_color_escape(out, color); last = Some(color); }
        out.push(ch);
      }
    } else {
      let mut last: Option<(u8, u8, u8)> = None;
      for col in 0..w {
        let level = grid[base + col];
        let ch = density_char(level, thresholds);
        let rgb = palette_sample(theme, clamp(level * bright));
        if Some(rgb) != last { write_rgb_escape(out, rgb); last = Some(rgb); }
        out.push(ch);
      }
    }
    out.push_str(RESET);
    // Raw mode disables OPOST/ONLCR, so a bare LF moves down without resetting
    // the column. Emit CR+LF so each row starts at column 1.
    if row + 1 < h { out.push_str("\r\n"); }
  }
}

/// Like `render_field` but each cell's glyph is given explicitly. Level is
/// used only for the colour; a level <= 0.001 emits a literal space and
/// resets the colour run.
pub fn render_glyph_field(
  ctx: &FrameContext,
  grid: &[f64],
  glyphs: &[char],
  style: &FieldStyle,
  out: &mut String,
) {
  let theme = resolve_theme(&ctx.options.theme, style.default_theme);
  let bright = ctx.options.brightness;
  let w = ctx.width;
  let h = ctx.height;
  let is_mono = theme == "mono";

  for row in 0..h {
    out.push_str(BLACK_BG);
    let base = row * w;
    if is_mono {
      let mut last: Option<u8> = None;
      for col in 0..w {
        let level = grid[base + col];
        if level <= 0.001 { out.push(' '); continue; }
        let color = gray_fg(level, style.gray_lo, style.gray_hi, bright);
        if Some(color) != last { write_color_escape(out, color); last = Some(color); }
        out.push(glyphs[base + col]);
      }
    } else {
      let mut last: Option<(u8, u8, u8)> = None;
      for col in 0..w {
        let level = grid[base + col];
        if level <= 0.001 { out.push(' '); continue; }
        let rgb = palette_sample(theme, clamp(level * bright));
        if Some(rgb) != last { write_rgb_escape(out, rgb); last = Some(rgb); }
        out.push(glyphs[base + col]);
      }
    }
    out.push_str(RESET);
    // Raw mode disables OPOST/ONLCR, so a bare LF moves down without resetting
    // the column. Emit CR+LF so each row starts at column 1.
    if row + 1 < h { out.push_str("\r\n"); }
  }
}

/// Solid coloured background blocks (wave-plane look). 24-step gray ramp.
pub fn render_block_field(ctx: &FrameContext, grid: &[f64], out: &mut String) {
  let theme = resolve_theme(&ctx.options.theme, "mono");
  let bright = ctx.options.brightness;
  let w = ctx.width;
  let h = ctx.height;
  let is_mono = theme == "mono";
  for row in 0..h {
    let base = row * w;
    if is_mono {
      let mut last: Option<u8> = None;
      for col in 0..w {
        let level = grid[base + col];
        let v = clamp(level * bright);
        let gray = 232 + (v * 23.0).round() as u8;
        if Some(gray) != last {
          let _ = write!(out, "\x1b[48;5;{}m", gray); last = Some(gray);
        }
        out.push(' ');
      }
    } else {
      let mut last: Option<(u8, u8, u8)> = None;
      for col in 0..w {
        let level = grid[base + col];
        let rgb = palette_sample(theme, clamp(level * bright));
        if Some(rgb) != last {
          let _ = write!(out, "\x1b[48;2;{};{};{}m", rgb.0, rgb.1, rgb.2);
          last = Some(rgb);
        }
        out.push(' ');
      }
    }
    out.push_str(RESET);
    // Raw mode disables OPOST/ONLCR, so a bare LF moves down without resetting
    // the column. Emit CR+LF so each row starts at column 1.
    if row + 1 < h { out.push_str("\r\n"); }
  }
}
