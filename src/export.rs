//! Offline exporters for deterministic clips.

use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::PathBuf;

use font8x8::{UnicodeFonts, BASIC_FONTS};
use gif::{Encoder, Frame, Repeat};
use serde_json::json;

use crate::animation::{FrameContext, THEME_COLOR_STEPS};
use crate::options::{normalize_charset, RenderOptions};
use crate::playlist::Playlist;
use crate::registry;

const DEFAULT_FG: Rgb = (240, 240, 240);
const DEFAULT_BG: Rgb = (0, 0, 0);
const CELL_W: usize = 8;
const CELL_H: usize = 10;
const FONT_Y: usize = 1;

type Rgb = (u8, u8, u8);

pub enum ExportFormat {
  Gif,
  Asciinema,
}

pub struct ExportConfig {
  pub format: ExportFormat,
  pub path: PathBuf,
  pub fps: f64,
  pub seconds: f64,
  pub width: usize,
  pub height: usize,
  pub options: RenderOptions,
  pub mode_options: BTreeMap<String, RenderOptions>,
}

struct Active {
  options: RenderOptions,
}

#[derive(Clone, Copy)]
struct Cell {
  ch: char,
  fg: Rgb,
  bg: Rgb,
}

impl Default for Cell {
  fn default() -> Self {
    Self { ch: ' ', fg: DEFAULT_FG, bg: DEFAULT_BG }
  }
}

pub fn export(mut playlist: Playlist, cfg: ExportConfig) -> io::Result<()> {
  let fps = cfg.fps.max(1.0);
  let seconds = cfg.seconds.max(1.0 / fps);
  let width = cfg.width.max(1);
  let height = cfg.height.max(1);
  let frame_count = (seconds * fps).ceil().max(1.0) as usize;
  let frame_dt = 1.0 / fps;

  let mut mode_options = cfg.mode_options.clone();
  let initial_name = playlist.name().to_string();
  if !mode_options.contains_key(&initial_name) {
    mode_options.insert(initial_name.clone(), cfg.options.clone());
  }
  let mut active = Active { options: mode_options[&initial_name].clone() };
  enforce_charset_support(&initial_name, &mut active.options);

  let mut current_name = String::new();
  let mut virtual_t = 0.0_f64;
  let mut frame_buf = String::with_capacity(width * height * 12);

  match cfg.format {
    ExportFormat::Gif => {
      let mut sink = GifSink::create(cfg.path, width, height, fps)?;
      render_frames(
        &mut playlist,
        &mut active,
        &mode_options,
        &mut current_name,
        width,
        height,
        frame_count,
        frame_dt,
        &mut virtual_t,
        &mut frame_buf,
        |_, frame| sink.write_frame(frame),
      )?;
      sink.finish()
    }
    ExportFormat::Asciinema => {
      let mut sink = CastSink::create(cfg.path, width, height)?;
      render_frames(
        &mut playlist,
        &mut active,
        &mode_options,
        &mut current_name,
        width,
        height,
        frame_count,
        frame_dt,
        &mut virtual_t,
        &mut frame_buf,
        |time, frame| sink.write_frame(time, frame),
      )?;
      sink.finish(seconds)
    }
  }
}

#[allow(clippy::too_many_arguments)]
fn render_frames<F>(
  playlist: &mut Playlist,
  active: &mut Active,
  mode_options: &BTreeMap<String, RenderOptions>,
  current_name: &mut String,
  width: usize,
  height: usize,
  frame_count: usize,
  frame_dt: f64,
  virtual_t: &mut f64,
  frame_buf: &mut String,
  mut on_frame: F,
) -> io::Result<()>
where
  F: FnMut(f64, &str) -> io::Result<()>,
{
  for idx in 0..frame_count {
    playlist.maybe_advance(*virtual_t);
    let name = playlist.name();
    if name != current_name.as_str() {
      *current_name = name.to_string();
      active.options =
        mode_options.get(current_name.as_str()).cloned().unwrap_or_else(RenderOptions::default);
      enforce_charset_support(current_name.as_str(), &mut active.options);
    }

    let elapsed_scene = playlist.scene_elapsed(*virtual_t);
    let phase = (elapsed_scene % 24.0) / 24.0;
    frame_buf.clear();
    let ctx = FrameContext {
      width,
      height,
      elapsed: elapsed_scene,
      phase,
      color_steps: THEME_COLOR_STEPS,
      options: &active.options,
    };
    playlist.current().render(&ctx, frame_buf);
    on_frame(idx as f64 * frame_dt, frame_buf.as_str())?;

    *virtual_t += frame_dt * active.options.speed;
  }
  Ok(())
}

fn enforce_charset_support(mode: &str, options: &mut RenderOptions) {
  options.charset = normalize_charset(&options.charset);
  if !registry::supports_charset(mode) {
    options.charset = "scene".to_string();
  }
}

struct CastSink {
  writer: BufWriter<File>,
}

impl CastSink {
  fn create(path: PathBuf, width: usize, height: usize) -> io::Result<Self> {
    let mut writer = BufWriter::new(File::create(path)?);
    write_json_line(
      &mut writer,
      &json!({
        "version": 2,
        "width": width,
        "height": height,
        "env": { "TERM": "xterm-256color", "SHELL": "ascii-fields" },
      }),
    )?;
    Ok(Self { writer })
  }

  fn write_frame(&mut self, time: f64, frame: &str) -> io::Result<()> {
    let mut data = String::with_capacity(frame.len() + 32);
    if time == 0.0 {
      data.push_str("\x1b[?25l\x1b[2J");
    }
    data.push_str("\x1b[H");
    data.push_str(frame);
    data.push_str("\x1b[0m");
    write_json_line(&mut self.writer, &json!([time, "o", data]))
  }

  fn finish(&mut self, seconds: f64) -> io::Result<()> {
    write_json_line(&mut self.writer, &json!([seconds, "o", "\x1b[0m\x1b[?25h"]))?;
    self.writer.flush()
  }
}

fn write_json_line<W: Write, T: serde::Serialize>(writer: &mut W, value: &T) -> io::Result<()> {
  serde_json::to_writer(&mut *writer, value).map_err(io_other)?;
  writer.write_all(b"\n")
}

struct GifSink {
  encoder: Encoder<BufWriter<File>>,
  width: usize,
  height: usize,
  pixel_w: usize,
  pixel_h: usize,
  delay_cs: u16,
}

impl GifSink {
  fn create(path: PathBuf, width: usize, height: usize, fps: f64) -> io::Result<Self> {
    let pixel_w = width.checked_mul(CELL_W).ok_or_else(|| io_invalid("GIF width is too large"))?;
    let pixel_h = height.checked_mul(CELL_H).ok_or_else(|| io_invalid("GIF height is too large"))?;
    if pixel_w > u16::MAX as usize || pixel_h > u16::MAX as usize {
      return Err(io_invalid("GIF dimensions exceed 65535 pixels"));
    }

    let writer = BufWriter::new(File::create(path)?);
    let mut encoder = Encoder::new(writer, pixel_w as u16, pixel_h as u16, &[]).map_err(io_other)?;
    encoder.set_repeat(Repeat::Infinite).map_err(io_other)?;

    Ok(Self {
      encoder,
      width,
      height,
      pixel_w,
      pixel_h,
      delay_cs: (100.0 / fps.max(1.0)).round().clamp(1.0, u16::MAX as f64) as u16,
    })
  }

  fn write_frame(&mut self, ansi_frame: &str) -> io::Result<()> {
    let cells = parse_ansi_cells(ansi_frame, self.width, self.height);
    let pixels = rasterize(&cells, self.width, self.height, self.pixel_w, self.pixel_h);
    let mut frame = Frame::from_rgb(self.pixel_w as u16, self.pixel_h as u16, &pixels);
    frame.delay = self.delay_cs;
    self.encoder.write_frame(&frame).map_err(io_other)
  }

  fn finish(&mut self) -> io::Result<()> {
    self.encoder.get_mut().flush()
  }
}

fn parse_ansi_cells(input: &str, width: usize, height: usize) -> Vec<Cell> {
  let mut cells = vec![Cell::default(); width * height];
  let chars: Vec<char> = input.chars().collect();
  let mut row = 0_usize;
  let mut col = 0_usize;
  let mut fg = DEFAULT_FG;
  let mut bg = DEFAULT_BG;
  let mut idx = 0_usize;

  while idx < chars.len() {
    match chars[idx] {
      '\x1b' if idx + 1 < chars.len() && chars[idx + 1] == '[' => {
        let mut end = idx + 2;
        while end < chars.len() && chars[end] != 'm' {
          end += 1;
        }
        if end < chars.len() {
          let params: String = chars[idx + 2..end].iter().collect();
          apply_sgr(&params, &mut fg, &mut bg);
          idx = end + 1;
          continue;
        }
      }
      '\n' => {
        row += 1;
        col = 0;
        idx += 1;
        continue;
      }
      '\r' => {
        col = 0;
        idx += 1;
        continue;
      }
      ch => {
        if row < height && col < width {
          cells[row * width + col] = Cell { ch, fg, bg };
        }
        col += 1;
        idx += 1;
        continue;
      }
    }
    idx += 1;
  }

  cells
}

fn apply_sgr(params: &str, fg: &mut Rgb, bg: &mut Rgb) {
  if params.is_empty() {
    *fg = DEFAULT_FG;
    *bg = DEFAULT_BG;
    return;
  }

  let codes: Vec<i32> = params
    .split(';')
    .map(|part| if part.is_empty() { 0 } else { part.parse::<i32>().unwrap_or(0) })
    .collect();
  let mut idx = 0_usize;

  while idx < codes.len() {
    match codes[idx] {
      0 => {
        *fg = DEFAULT_FG;
        *bg = DEFAULT_BG;
      }
      30..=37 => *fg = ansi_16((codes[idx] - 30) as u8),
      40..=47 => *bg = ansi_16((codes[idx] - 40) as u8),
      90..=97 => *fg = ansi_16((codes[idx] - 90 + 8) as u8),
      100..=107 => *bg = ansi_16((codes[idx] - 100 + 8) as u8),
      39 => *fg = DEFAULT_FG,
      49 => *bg = DEFAULT_BG,
      38 | 48 => {
        let is_fg = codes[idx] == 38;
        if idx + 2 < codes.len() && codes[idx + 1] == 5 {
          let color = xterm_256(codes[idx + 2].clamp(0, 255) as u8);
          if is_fg {
            *fg = color;
          } else {
            *bg = color;
          }
          idx += 2;
        } else if idx + 4 < codes.len() && codes[idx + 1] == 2 {
          let color = (
            codes[idx + 2].clamp(0, 255) as u8,
            codes[idx + 3].clamp(0, 255) as u8,
            codes[idx + 4].clamp(0, 255) as u8,
          );
          if is_fg {
            *fg = color;
          } else {
            *bg = color;
          }
          idx += 4;
        }
      }
      _ => {}
    }
    idx += 1;
  }
}

fn rasterize(cells: &[Cell], width: usize, height: usize, pixel_w: usize, pixel_h: usize) -> Vec<u8> {
  let mut pixels = vec![0_u8; pixel_w * pixel_h * 3];

  for row in 0..height {
    for col in 0..width {
      let cell = cells[row * width + col];
      fill_cell(&mut pixels, pixel_w, row, col, cell.bg);
      draw_glyph(&mut pixels, pixel_w, row, col, cell.ch, cell.fg);
    }
  }

  pixels
}

fn fill_cell(pixels: &mut [u8], pixel_w: usize, row: usize, col: usize, color: Rgb) {
  let x0 = col * CELL_W;
  let y0 = row * CELL_H;
  for y in y0..(y0 + CELL_H) {
    for x in x0..(x0 + CELL_W) {
      put_pixel(pixels, pixel_w, x, y, color);
    }
  }
}

fn draw_glyph(pixels: &mut [u8], pixel_w: usize, row: usize, col: usize, ch: char, color: Rgb) {
  if ch == ' ' {
    return;
  }
  let glyph = BASIC_FONTS.get(ch).or_else(|| BASIC_FONTS.get('?'));
  let Some(bitmap) = glyph else {
    return;
  };
  let x0 = col * CELL_W;
  let y0 = row * CELL_H + FONT_Y;
  for (gy, bits) in bitmap.iter().enumerate() {
    for gx in 0..8 {
      if bits & (1 << gx) != 0 {
        put_pixel(pixels, pixel_w, x0 + gx, y0 + gy, color);
      }
    }
  }
}

fn put_pixel(pixels: &mut [u8], pixel_w: usize, x: usize, y: usize, color: Rgb) {
  let idx = (y * pixel_w + x) * 3;
  pixels[idx] = color.0;
  pixels[idx + 1] = color.1;
  pixels[idx + 2] = color.2;
}

fn ansi_16(idx: u8) -> Rgb {
  const COLORS: [Rgb; 16] = [
    (0, 0, 0),
    (205, 0, 0),
    (0, 205, 0),
    (205, 205, 0),
    (0, 0, 238),
    (205, 0, 205),
    (0, 205, 205),
    (229, 229, 229),
    (127, 127, 127),
    (255, 0, 0),
    (0, 255, 0),
    (255, 255, 0),
    (92, 92, 255),
    (255, 0, 255),
    (0, 255, 255),
    (255, 255, 255),
  ];
  COLORS[idx as usize]
}

fn xterm_256(idx: u8) -> Rgb {
  if idx < 16 {
    return ansi_16(idx);
  }
  if idx < 232 {
    let n = idx - 16;
    let scale = [0, 95, 135, 175, 215, 255];
    return (scale[(n / 36) as usize], scale[((n / 6) % 6) as usize], scale[(n % 6) as usize]);
  }
  let gray = 8 + (idx - 232) * 10;
  (gray, gray, gray)
}

fn io_other<E: std::fmt::Display>(err: E) -> io::Error {
  io::Error::other(err.to_string())
}

fn io_invalid(message: &str) -> io::Error {
  io::Error::new(io::ErrorKind::InvalidInput, message)
}
