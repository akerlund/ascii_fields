//! Frame loop: alt-screen, raw mode, key polling, HUD pinned to
//! the bottom rows, pause (skip render), settings save/load.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::io::{self, BufWriter, Write};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use cpu_time::ProcessTime;
use crossterm::{
  cursor::{Hide, Show},
  event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
  execute, queue,
  terminal::{
    self, disable_raw_mode, enable_raw_mode, Clear, ClearType, DisableLineWrap, EnableLineWrap,
    EnterAlternateScreen, LeaveAlternateScreen,
  },
};

use crate::animation::{FrameContext, THEME_COLOR_STEPS};
use crate::options::{normalize_charset, RenderOptions, CHARSET_CYCLE};
use crate::playlist::Playlist;
use crate::registry;
use crate::settings;
use crate::themes::THEME_CYCLE;

pub struct RunConfig {
  pub fps: f64,
  pub seconds: f64,
  pub width: Option<usize>,
  pub height: Option<usize>,
  pub options: RenderOptions,
  pub mode_options: BTreeMap<String, RenderOptions>,
  pub saved_mode_options: BTreeMap<String, RenderOptions>,
  pub favorites: Vec<String>,
  pub no_status: bool,
  pub settings_path: PathBuf,
}

struct Active {
  options: RenderOptions,
}

pub fn run(mut playlist: Playlist, cfg: RunConfig) -> io::Result<()> {
  let frame_time = Duration::from_secs_f64(1.0 / cfg.fps.max(1.0));
  // BufWriter so each frame goes out as one write+flush rather than the
  // line-buffered stdout flushing on every '\n'.
  let mut stdout = BufWriter::with_capacity(128 * 1024, io::stdout().lock());

  enable_raw_mode().ok();
  execute!(stdout, EnterAlternateScreen, Hide, Clear(ClearType::All))?;

  let started = Instant::now();
  let mut last_wall = started;
  let mut next_frame = started;
  let mut virtual_t = 0.0_f64;
  let mut paused = false;
  let mut hud_visible = !cfg.no_status;
  let mut last_frame = String::with_capacity(64 * 1024);
  let mut draw_buf = String::with_capacity(96 * 1024);
  let mut hud_buf = String::new();
  let mut save_msg = String::new();
  let mut save_until = Instant::now();
  let mut favorites = cfg.favorites.clone();
  let mut fps_actual = cfg.fps;
  let mut fps_frames = 0_u32;
  let mut prev_fps_wall = Instant::now();
  let mut cpu_pct = 0.0_f64;
  let mut prev_proc = ProcessTime::now();
  let mut prev_wall = Instant::now();
  let mut last_dims = (0_u16, 0_u16);

  let mut mode_options = cfg.mode_options.clone();
  let mut saved_mode_options = cfg.saved_mode_options.clone();
  // Use the entry for the starting mode, or fall back to the CLI-built default.
  let initial_name = playlist.name().to_string();
  if !mode_options.contains_key(&initial_name) {
    mode_options.insert(initial_name.clone(), cfg.options.clone());
  }
  if !saved_mode_options.contains_key(&initial_name) {
    let mut options = RenderOptions::default();
    enforce_charset_support(&initial_name, &mut options);
    saved_mode_options.insert(initial_name.clone(), options);
  }
  let mut current_name = initial_name.clone();
  let mut active = Active { options: mode_options[&initial_name].clone() };
  enforce_charset_support(&initial_name, &mut active.options);
  mode_options.insert(initial_name.clone(), active.options.clone());

  let result = (|| -> io::Result<()> {
    execute!(stdout, DisableLineWrap)?;
    loop {
      let now = Instant::now();
      if cfg.seconds > 0.0 && now.duration_since(started).as_secs_f64() >= cfg.seconds {
        break;
      }

      // virtual time advances with speed; pauses freeze it
      let dt = now.duration_since(last_wall).as_secs_f64();
      last_wall = now;
      if !paused {
        virtual_t += dt * active.options.speed;
      }

      // input
      while event::poll(Duration::ZERO)? {
        if let Event::Key(KeyEvent { code, modifiers, kind, .. }) = event::read()? {
          if kind == KeyEventKind::Release {
            continue;
          }
          match (code, modifiers) {
            (KeyCode::Char('q'), _) | (KeyCode::Esc, _) => return Ok(()),
            (KeyCode::Char('c'), m) if m.contains(KeyModifiers::CONTROL) => return Ok(()),
            (KeyCode::Char(' '), _) => paused = !paused,
            (KeyCode::Char('+'), _) | (KeyCode::Char('='), _) => {
              active.options.speed = (active.options.speed * 1.25).min(8.0);
            }
            (KeyCode::Char('-'), _) | (KeyCode::Char('_'), _) => {
              active.options.speed = (active.options.speed * 0.8).max(0.1);
            }
            (KeyCode::Char('n'), _) | (KeyCode::Tab, _) => {
              playlist.go_next(virtual_t);
            }
            (KeyCode::Char('p'), _) => {
              playlist.go_prev(virtual_t);
            }
            (KeyCode::Char('r'), _) => playlist.restart(virtual_t),
            (KeyCode::Char('s'), _) => {
              enforce_charset_support(&current_name, &mut active.options);
              mode_options.insert(current_name.clone(), active.options.clone());
              saved_mode_options.insert(current_name.clone(), active.options.clone());
              match settings::save(&cfg.settings_path, &saved_mode_options, &favorites) {
                Ok(()) => save_msg = format!("saved {}", cfg.settings_path.display()),
                Err(e) => save_msg = format!("save error: {}", e),
              }
              save_until = Instant::now() + Duration::from_millis(2500);
            }
            (KeyCode::Char('f'), _) => {
              let favorite_name = playlist.name().to_string();
              if favorites.iter().any(|name| name == &favorite_name) {
                save_msg = format!("already favorite {}", favorite_name);
              } else {
                favorites.push(favorite_name.clone());
                match settings::save(&cfg.settings_path, &saved_mode_options, &favorites) {
                  Ok(()) => save_msg = format!("favorited {}", favorite_name),
                  Err(e) => save_msg = format!("favorite error: {}", e),
                }
              }
              save_until = Instant::now() + Duration::from_millis(2500);
            }
            (KeyCode::Char('i'), _) => {
              hud_visible = !hud_visible;
              last_dims = (0, 0); // force a repaint at the new size
            }
            (KeyCode::Char('t'), _) => cycle_theme(&mut active.options.theme, 1),
            (KeyCode::Char('T'), _) => cycle_theme(&mut active.options.theme, -1),
            (KeyCode::Char('c'), _) => {
              if registry::supports_charset(playlist.name()) {
                cycle_charset(&mut active.options.charset);
              } else {
                active.options.charset = "scene".to_string();
                save_msg = format!("charset locked for {}", playlist.name());
                save_until = Instant::now() + Duration::from_millis(2500);
              }
            }
            (KeyCode::Char('1'), _) => step_param(&mut active.options.scale, 0.91),
            (KeyCode::Char('2'), _) => step_param(&mut active.options.scale, 1.10),
            (KeyCode::Char('3'), _) => step_param(&mut active.options.contrast, 0.91),
            (KeyCode::Char('4'), _) => step_param(&mut active.options.contrast, 1.10),
            (KeyCode::Char('5'), _) => step_param(&mut active.options.brightness, 0.91),
            (KeyCode::Char('6'), _) => step_param(&mut active.options.brightness, 1.10),
            _ => {}
          }
        }
      }

      // auto-advance for random/cycle playlists
      playlist.maybe_advance(virtual_t);
      let name = playlist.name();
      if name != current_name {
        current_name = name.to_string();
        active.options =
          mode_options.get(current_name.as_str()).cloned().unwrap_or_else(|| cfg.options.clone());
        enforce_charset_support(&current_name, &mut active.options);
        last_dims = (0, 0);
      }

      // dimensions + HUD reserve
      let (cols, rows) = terminal::size().unwrap_or((100, 40));
      let hud_lines: u16 = if hud_visible { 4 } else { 0 };
      let drawable_cols = cols.max(1) as usize;
      let drawable_rows = rows.saturating_sub(hud_lines).max(1) as usize;
      let cw = cfg.width.unwrap_or(drawable_cols).clamp(1, drawable_cols);
      let ch = cfg.height.unwrap_or(drawable_rows).clamp(1, drawable_rows);

      if last_dims != (cols, rows) {
        queue!(stdout, Clear(ClearType::All))?;
        last_dims = (cols, rows);
      }

      // render (skip when paused, reuse last_frame)
      let elapsed_scene = playlist.scene_elapsed(virtual_t);
      let phase = (elapsed_scene % 24.0) / 24.0;
      if !paused {
        last_frame.clear();
        let ctx = FrameContext {
          width: cw,
          height: ch,
          elapsed: elapsed_scene,
          phase,
          color_steps: THEME_COLOR_STEPS,
          options: &active.options,
        };
        playlist.current().render(&ctx, &mut last_frame);
      }

      if !paused {
        fps_frames += 1;
      }

      // process CPU usage, sampled once per 500ms (= updates 2x / second).
      // 100% == one full core. Less jitter than per-frame EMA.
      let now_wall = Instant::now();
      if now_wall.duration_since(prev_fps_wall) >= Duration::from_millis(500) {
        let elapsed = now_wall.duration_since(prev_fps_wall).as_secs_f64();
        if elapsed > 0.0 {
          fps_actual = fps_frames as f64 / elapsed;
        }
        fps_frames = 0;
        prev_fps_wall = now_wall;
      }
      if now_wall.duration_since(prev_wall) >= Duration::from_millis(500) {
        let now_proc = ProcessTime::now();
        let d_proc = now_proc.duration_since(prev_proc).as_secs_f64();
        let d_wall = now_wall.duration_since(prev_wall).as_secs_f64();
        if d_wall > 0.0 {
          cpu_pct = (d_proc / d_wall) * 100.0;
        }
        prev_proc = now_proc;
        prev_wall = now_wall;
      }

      // HUD rows with vertical column separators. Each cell is a
      // fixed width so the `|` markers line up across rows.
      hud_buf.clear();
      if hud_visible {
        let state = if paused {
          "PAUSED".to_string()
        } else if (active.options.speed - 1.0).abs() > 0.02 {
          format!("x{:.2}", active.options.speed)
        } else {
          "play".to_string()
        };
        let shown_state =
          if !save_msg.is_empty() && Instant::now() < save_until { save_msg.clone() } else { state };
        let line1 = format!(
          "{}|{}|{}|{}|{}|{}| {}",
          cell(&format!(" {}", playlist.title()), 22),
          cell(&format!(" theme = {}", active.options.theme), 20),
          cell(&format!(" scale = {:>4.2}", active.options.scale), 15),
          cell(&format!(" contrast = {:>4.2}", active.options.contrast), 17),
          cell(&format!(" Bright = {:>4.2}", active.options.brightness), 14),
          cell(&format!(" Charset = {}", active.options.charset), 18),
          shown_state,
        );
        let line2 = format!(
          "{}|{}|{}|{}|{}|{}|{}",
          cell(" [n/p]switch", 22),
          cell(" [t/T]", 20),
          cell(" [1/2]", 15),
          cell(" [3/4]", 17),
          cell(" [5/6]", 14),
          cell(" [c]", 18),
          " [+/-]",
        );
        let line3 = format!(
          "{}|{}|{}|{}|{}|{}|{}",
          cell(" [i] Menu [q]quit", 22),
          cell(" [s]save", 20),
          cell(" [f] Favorite", 15),
          cell("", 17),
          cell("", 14),
          cell("", 18),
          " [_]pause",
        );
        let line4 = format!(
          "{}|{}|{}|{}|{}|{}|{}",
          cell(&format!(" cpu = {:>5.1}%", cpu_pct), 22),
          cell(&format!(" fps = {:>2.0}/{:<2.0}", fps_actual, cfg.fps), 20),
          cell("", 15),
          cell("", 17),
          cell("", 14),
          cell("", 18),
          "",
        );

        let row1 = rows.saturating_sub(4);
        let row2 = rows.saturating_sub(3);
        let row3 = rows.saturating_sub(2);
        let row4 = rows.saturating_sub(1);
        let _ = write!(
          hud_buf,
          "\x1b[{};1H\x1b[48;2;0;0;0m{}\x1b[0m\
           \x1b[{};1H\x1b[48;2;0;0;0m{}\x1b[0m\
           \x1b[{};1H\x1b[48;2;0;0;0m{}\x1b[0m\
           \x1b[{};1H\x1b[48;2;0;0;0m{}\x1b[0m",
          row1 + 1,
          fit(&line1, cw),
          row2 + 1,
          fit(&line2, cw),
          row3 + 1,
          fit(&line3, cw),
          row4 + 1,
          fit(&line4, cw),
        );
      }

      // write
      if !paused {
        draw_buf.clear();
        for (row, line) in last_frame.lines().take(ch).enumerate() {
          let _ = write!(draw_buf, "\x1b[{};1H{}", row + 1, line);
        }
        stdout.write_all(draw_buf.as_bytes())?;
      }
      stdout.write_all(hud_buf.as_bytes())?;
      stdout.flush()?;

      // sleep
      next_frame += frame_time;
      let now2 = Instant::now();
      if next_frame > now2 {
        std::thread::sleep(next_frame - now2);
      } else {
        next_frame = now2;
      }
    }
    Ok(())
  })();

  execute!(stdout, EnableLineWrap, LeaveAlternateScreen, Show, crossterm::style::ResetColor)?;
  disable_raw_mode().ok();
  result
}

fn fit(text: &str, width: usize) -> String {
  // Truncate OR pad with spaces so the result is exactly `width` columns.
  // Padding (rather than relying on \x1b[K) is what guarantees the black HUD
  // background fills every cell of the row -- some terminals don't honour the
  // current SGR background for \x1b[K reliably (especially under tmux).
  let n = text.len();
  if n >= width {
    text[..width].to_string()
  } else {
    let mut s = String::with_capacity(width);
    s.push_str(text);
    for _ in 0..(width - n) {
      s.push(' ');
    }
    s
  }
}

/// Trim or right-pad `content` so it occupies exactly `width` columns. ASCII
/// only, so char count == byte count for our HUD strings.
fn cell(content: &str, width: usize) -> String {
  let n = content.chars().count();
  if n >= width {
    content.chars().take(width).collect()
  } else {
    let mut s = String::with_capacity(width);
    s.push_str(content);
    for _ in 0..(width - n) {
      s.push(' ');
    }
    s
  }
}

fn step_param(v: &mut f64, factor: f64) {
  *v = (*v * factor).clamp(0.1, 5.0);
}

fn cycle_theme(name: &mut String, direction: i32) {
  let idx = THEME_CYCLE.iter().position(|t| *t == name.as_str()).unwrap_or(0) as i32;
  let n = THEME_CYCLE.len() as i32;
  let next = ((idx + direction).rem_euclid(n)) as usize;
  *name = THEME_CYCLE[next].to_string();
}

fn cycle_charset(name: &mut String) {
  let current = normalize_charset(name);
  let idx = CHARSET_CYCLE.iter().position(|c| *c == current.as_str()).unwrap_or(0);
  let next = (idx + 1) % CHARSET_CYCLE.len();
  *name = CHARSET_CYCLE[next].to_string();
}

fn enforce_charset_support(mode: &str, options: &mut RenderOptions) {
  options.charset = normalize_charset(&options.charset);
  if !registry::supports_charset(mode) {
    options.charset = "scene".to_string();
  }
}
