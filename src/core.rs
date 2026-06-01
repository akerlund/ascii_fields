//! Field renderers and shared math helpers. Three renderers:
//!
//! * [`render_field`] – the workhorse, glyph chosen from a level/threshold
//!   ramp or a shared charset override, foreground colour from a 256-color
//!   gray ramp or a truecolor theme.
//! * [`render_glyph_field`] – explicit per-cell glyph, level only used for
//!   the colour.

use crate::animation::FrameContext;
use crate::themes::{palette_table, STEPS as PALETTE_STEPS};

pub const BLACK_BG: &str = "\x1b[48;5;232m";
pub const RESET: &str = "\x1b[0m";

#[inline]
pub fn clamp(v: f64) -> f64 {
  v.clamp(0.0, 1.0)
}

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
  (0.12, ' '),
  (0.20, '.'),
  (0.30, ':'),
  (0.41, '-'),
  (0.53, '='),
  (0.66, '+'),
  (0.79, '*'),
  (0.91, '#'),
  (1.01, '%'),
];

// ASCII / classic ramps -----------------------------------------------------
const CLEAN_RAMP: &[char] = &[
  ' ', '.', '\'', '`', '^', '"', ',', ':', ';', 'I', 'l', '!', 'i', '~', '+', '_', '-', '?', ']', '[', '}',
  '{', '1', ')', '(', '|', '\\', '/', 't', 'f', 'j', 'r', 'x', 'n', 'u', 'v', 'c', 'z', 'X', 'Y', 'U', 'J',
  'C', 'L', 'Q', '0', 'O', 'Z', 'm', 'w', 'q', 'p', 'd', 'b', 'k', 'h', 'a', 'o', '*', '#', 'M', 'W', '&',
  '8', '%', 'B', '@', '$',
];
const SOFT_RAMP: &[char] = &[
  ' ', '.', '\'', '`', '^', '"', ',', ':', '-', '_', '~', '+', 'i', 't', 'o', '+', 'x', 'z', 'M', 'W', '#',
  '@', '$', '8',
];
const DENSE_RAMP: &[char] = &[
  ' ', '.', ',', ':', ';', 'i', 'r', 's', 'X', 'A', '2', '5', '3', 'h', 'M', 'H', 'G', 'S', '#', '9', 'B',
  '&', '@',
];
const MINIMAL_RAMP: &[char] = &[' ', '.', ':', '-', '#', '@'];

// Smooth: low chatter -- gradual ASCII steps, good for waves / ocean / heat
const SMOOTH_RAMP: &[char] = &[' ', '.', ',', ':', ';', '-', '=', '+', '*', '#', '%', '@'];

// Sharp: hard-edged technical, uses Unicode lower-block partials for crisp
// stepped contours -- good for schlieren, magnetic, reconnection
const SHARP_RAMP: &[char] = &[' ', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

// Matrix: digital glyph ramp, good for rain / cpu / network
const MATRIX_RAMP: &[char] = &[' ', '.', '0', '1', '0', '1', '2', '3', '5', '7', '8', '9', '#', '@', '$'];

// Braille: 2x4 dot patterns, higher apparent resolution -- good for
// fractals, attractors, dense fields. Needs a Unicode-capable terminal.
const BRAILLE_RAMP: &[char] = &[' ', '⠁', '⠃', '⠇', '⡇', '⡏', '⡟', '⡿', '⣿'];

#[inline]
pub fn density_char(level: f64, thresholds: &[(f64, char)]) -> char {
  for &(limit, ch) in thresholds {
    if level < limit {
      return ch;
    }
  }
  thresholds.last().map(|&(_, ch)| ch).unwrap_or(' ')
}

#[inline]
fn ramp_char(level: f64, ramp: &[char]) -> char {
  if ramp.is_empty() {
    return ' ';
  }
  let idx = (clamp(level) * ramp.len().saturating_sub(1) as f64).round() as usize;
  ramp[idx]
}

#[inline]
fn charset_char(level: f64, thresholds: &[(f64, char)], charset: &str) -> char {
  match charset {
    "clean" => ramp_char(level, CLEAN_RAMP),
    "soft" => ramp_char(level, SOFT_RAMP),
    "dense" => ramp_char(level, DENSE_RAMP),
    "minimal" => ramp_char(level, MINIMAL_RAMP),
    "smooth" => ramp_char(level, SMOOTH_RAMP),
    "sharp" => ramp_char(level, SHARP_RAMP),
    "matrix" => ramp_char(level, MATRIX_RAMP),
    "braille" => ramp_char(level, BRAILLE_RAMP),
    _ => density_char(level, thresholds),
  }
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
  fn default() -> Self {
    Self { gray_lo: 234, gray_hi: 255, default_theme: "mono" }
  }
}

/// Push the decimal representation of a u8 to a String -- ~3-5x faster than
/// the `format!`/`write!` machinery, which matters in tight per-cell loops.
#[inline]
fn push_u8(out: &mut String, n: u8) {
  if n >= 100 {
    out.push((b'0' + n / 100) as char);
    out.push((b'0' + (n / 10) % 10) as char);
    out.push((b'0' + n % 10) as char);
  } else if n >= 10 {
    out.push((b'0' + n / 10) as char);
    out.push((b'0' + n % 10) as char);
  } else {
    out.push((b'0' + n) as char);
  }
}

#[inline]
fn write_color_escape(out: &mut String, color: u8) {
  out.push_str("\x1b[38;5;");
  push_u8(out, color);
  out.push('m');
}

#[inline]
fn write_rgb_escape(out: &mut String, (r, g, b): (u8, u8, u8)) {
  out.push_str("\x1b[38;2;");
  push_u8(out, r);
  out.push(';');
  push_u8(out, g);
  out.push(';');
  push_u8(out, b);
  out.push('m');
}

#[inline]
fn write_bg_256_escape(out: &mut String, color: u8) {
  out.push_str("\x1b[48;5;");
  push_u8(out, color);
  out.push('m');
}

#[inline]
fn write_bg_rgb_escape(out: &mut String, (r, g, b): (u8, u8, u8)) {
  out.push_str("\x1b[48;2;");
  push_u8(out, r);
  out.push(';');
  push_u8(out, g);
  out.push(';');
  push_u8(out, b);
  out.push('m');
}

/// Precompute a 256-entry glyph LUT keyed by 8-bit brightness. Called once
/// per `render_field` invocation (so once per frame), replacing the per-cell
/// linear threshold scan / ramp index with a single array index.
fn build_glyph_lut(thresholds: &[(f64, char)], charset: &str) -> [char; 256] {
  let mut lut = [' '; 256];
  for i in 0..256 {
    let level = i as f64 / 255.0;
    lut[i] = charset_char(level, thresholds, charset);
  }
  lut
}

#[inline]
fn glyph_from_lut(lut: &[char; 256], level: f64) -> char {
  let idx = (clamp(level) * 255.0).round() as usize;
  lut[idx.min(255)]
}

#[inline]
fn color_level(level: f64, steps: usize) -> f64 {
  let value = clamp(level);
  let steps = steps.clamp(2, 256);
  if steps >= 256 {
    value
  } else {
    let last = (steps - 1) as f64;
    (value * last).round() / last
  }
}

/// Render a level grid; glyph comes from the thresholds.
pub fn render_field(
  ctx: &FrameContext,
  grid: &[f64],
  thresholds: &[(f64, char)],
  style: &FieldStyle,
  out: &mut String,
) {
  if ctx.options.charset == "blocks" {
    render_block_field_with_theme(ctx, grid, "mono", out);
    return;
  }

  let theme = resolve_theme(&ctx.options.theme, style.default_theme);
  let bright = ctx.options.brightness;
  let w = ctx.width;
  let h = ctx.height;
  let is_mono = theme == "mono";
  let charset = ctx.options.charset.as_str();
  // Fetch the palette ONCE per frame (themed path). Per-cell access becomes
  // a single array index instead of a mutex+hashmap+alloc.
  let palette: &[(u8, u8, u8)] = if is_mono { &[] } else { palette_table(theme) };
  let palette_max = PALETTE_STEPS.saturating_sub(1);
  // Precompute the brightness -> glyph LUT once per frame.
  let glyph_lut = build_glyph_lut(thresholds, charset);

  for row in 0..h {
    out.push_str(BLACK_BG);
    let base = row * w;
    if is_mono {
      let mut last: Option<u8> = None;
      for col in 0..w {
        let level = grid[base + col];
        let ch = glyph_from_lut(&glyph_lut, level);
        if ch == ' ' {
          out.push(' ');
          continue;
        }
        let color = gray_fg(level, style.gray_lo, style.gray_hi, bright);
        if Some(color) != last {
          write_color_escape(out, color);
          last = Some(color);
        }
        out.push(ch);
      }
    } else {
      let mut last: Option<(u8, u8, u8)> = None;
      for col in 0..w {
        let level = grid[base + col];
        let ch = glyph_from_lut(&glyph_lut, level);
        if ch == ' ' {
          out.push(' ');
          continue;
        }
        let q = color_level(level * bright, ctx.color_steps);
        let idx = (q * palette_max as f64 + 0.5) as usize;
        let rgb = palette[idx.min(palette_max)];
        if Some(rgb) != last {
          write_rgb_escape(out, rgb);
          last = Some(rgb);
        }
        out.push(ch);
      }
    }
    out.push_str(RESET);
    // Raw mode disables OPOST/ONLCR, so a bare LF moves down without resetting
    // the column. Emit CR+LF so each row starts at column 1.
    if row + 1 < h {
      out.push_str("\r\n");
    }
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
  let palette: &[(u8, u8, u8)] = if is_mono { &[] } else { palette_table(theme) };
  let palette_max = PALETTE_STEPS.saturating_sub(1);

  for row in 0..h {
    out.push_str(BLACK_BG);
    let base = row * w;
    if is_mono {
      let mut last: Option<u8> = None;
      for col in 0..w {
        let level = grid[base + col];
        if level <= 0.001 {
          out.push(' ');
          continue;
        }
        let color = gray_fg(level, style.gray_lo, style.gray_hi, bright);
        if Some(color) != last {
          write_color_escape(out, color);
          last = Some(color);
        }
        out.push(glyphs[base + col]);
      }
    } else {
      let mut last: Option<(u8, u8, u8)> = None;
      for col in 0..w {
        let level = grid[base + col];
        if level <= 0.001 {
          out.push(' ');
          continue;
        }
        let q = color_level(level * bright, ctx.color_steps);
        let idx = (q * palette_max as f64 + 0.5) as usize;
        let rgb = palette[idx.min(palette_max)];
        if Some(rgb) != last {
          write_rgb_escape(out, rgb);
          last = Some(rgb);
        }
        out.push(glyphs[base + col]);
      }
    }
    out.push_str(RESET);
    // Raw mode disables OPOST/ONLCR, so a bare LF moves down without resetting
    // the column. Emit CR+LF so each row starts at column 1.
    if row + 1 < h {
      out.push_str("\r\n");
    }
  }
}

fn render_block_field_with_theme(ctx: &FrameContext, grid: &[f64], default_theme: &str, out: &mut String) {
  let theme = resolve_theme(&ctx.options.theme, default_theme);
  let bright = ctx.options.brightness;
  let w = ctx.width;
  let h = ctx.height;
  let is_mono = theme == "mono";
  let palette: &[(u8, u8, u8)] = if is_mono { &[] } else { palette_table(theme) };
  let palette_max = PALETTE_STEPS.saturating_sub(1);
  for row in 0..h {
    let base = row * w;
    if is_mono {
      let mut last: Option<u8> = None;
      for col in 0..w {
        let level = grid[base + col];
        let v = clamp(level * bright);
        let gray = 232 + (v * 23.0).round() as u8;
        if Some(gray) != last {
          write_bg_256_escape(out, gray);
          last = Some(gray);
        }
        out.push(' ');
      }
    } else {
      let mut last: Option<(u8, u8, u8)> = None;
      for col in 0..w {
        let level = grid[base + col];
        let q = color_level(level * bright, ctx.color_steps);
        let idx = (q * palette_max as f64 + 0.5) as usize;
        let rgb = palette[idx.min(palette_max)];
        if Some(rgb) != last {
          write_bg_rgb_escape(out, rgb);
          last = Some(rgb);
        }
        out.push(' ');
      }
    }
    out.push_str(RESET);
    // Raw mode disables OPOST/ONLCR, so a bare LF moves down without resetting
    // the column. Emit CR+LF so each row starts at column 1.
    if row + 1 < h {
      out.push_str("\r\n");
    }
  }
}
