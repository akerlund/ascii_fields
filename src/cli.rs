//! Command-line surface and main entry. Matches the Python CLI flags.

use std::collections::BTreeMap;
use std::io::{self, ErrorKind};
use std::path::PathBuf;

use clap::Parser;

use crate::export::{self, ExportConfig, ExportFormat};
use crate::options::{normalize_charset, RenderOptions};
use crate::playlist::Playlist;
use crate::registry;
use crate::runner::{self, RunConfig};
use crate::settings;

#[derive(Parser, Debug)]
#[command(
  name = "ascii-fields",
  about = "Procedural ASCII animations for the terminal.",
  version
)]
pub struct Cli {
  /// Animation mode (use --list to see what's available).
  /// Use 'random' or 'cycle' for a playlist.
  pub mode: Option<String>,

  #[arg(long)] pub list: bool,
  /// Restrict --list, random, cycle, and scene switching to saved favorites.
  #[arg(long)] pub favorites: bool,
  #[arg(long)] pub width: Option<usize>,
  #[arg(long)] pub height: Option<usize>,
  #[arg(long, default_value_t = 24.0)] pub fps: f64,
  /// Per-scene duration for interactive playlists; total clip length for exports.
  #[arg(long, default_value_t = 0.0)] pub seconds: f64,
  #[arg(long)] pub scale: Option<f64>,
  #[arg(long)] pub contrast: Option<f64>,
  #[arg(long)] pub brightness: Option<f64>,
  #[arg(long)] pub speed: Option<f64>,
  #[arg(long)] pub theme: Option<String>,
  #[arg(long)] pub no_status: bool,
  #[arg(long, value_name = "scene|clean|soft|dense|minimal|blocks")] pub charset: Option<String>,
  #[arg(long, default_value_t = false)] pub scroll: bool,

  /// Export an animated GIF clip instead of running interactively.
  #[arg(long, value_name = "PATH")] pub export_gif: Option<PathBuf>,

  /// Export an asciinema v2 .cast clip instead of running interactively.
  #[arg(long, value_name = "PATH")] pub export_asciinema: Option<PathBuf>,
}

pub fn run() -> std::io::Result<()> {
  let cli = Cli::parse();
  let saved = settings::load(settings::DEFAULT_PATH);
  let favorites = known_favorites(&saved.favorites);

  if cli.list {
    print_mode_list(cli.favorites, &favorites);
    return Ok(());
  }

  let name = cli.mode.clone().unwrap_or_else(|| {
    if cli.favorites { "random".to_string() } else { "wave-plane".to_string() }
  });
  let favorite_filter = if cli.favorites {
    if favorites.is_empty() {
      return Err(no_favorites_error());
    }
    Some(favorites.as_slice())
  } else {
    None
  };

  let make_saved_options = |base_name: &str| -> RenderOptions {
    let mut opt = RenderOptions::default();
    if let Some(entry) = saved.modes.get(base_name) {
      entry.apply(&mut opt);
    }
    apply_charset_support(base_name, &mut opt);
    opt
  };

  let make_options = |base_name: &str| -> RenderOptions {
    let mut opt = make_saved_options(base_name);
    if let Some(v) = cli.scale { opt.scale = v; }
    if let Some(v) = cli.contrast { opt.contrast = v; }
    if let Some(v) = cli.brightness { opt.brightness = v; }
    if let Some(v) = cli.speed { opt.speed = v; }
    if let Some(ref v) = cli.theme { opt.theme = v.clone(); }
    opt.scroll = cli.scroll;
    if let Some(ref v) = cli.charset {
      opt.charset = normalize_charset(v);
    }
    apply_charset_support(base_name, &mut opt);
    opt
  };

  let saved_mode_options: BTreeMap<String, RenderOptions> = registry::MODES
    .iter()
    .map(|m| (m.name.to_string(), make_saved_options(m.name)))
    .collect();
  let mode_options: BTreeMap<String, RenderOptions> = registry::MODES
    .iter()
    .map(|m| (m.name.to_string(), make_options(m.name)))
    .collect();

  let playlist = make_playlist(&name, favorite_filter)?;
  let initial_options = mode_options
    .get(playlist.name())
    .cloned()
    .unwrap_or_else(|| make_options(playlist.name()));

  if let Some(path) = cli.export_gif.as_ref() {
    let playlist = make_playlist(&name, favorite_filter)?;
    export::export(playlist, ExportConfig {
      format: ExportFormat::Gif,
      path: path.clone(),
      fps: cli.fps,
      seconds: export_seconds(cli.seconds),
      width: cli.width.unwrap_or(100),
      height: cli.height.unwrap_or(40),
      options: initial_options.clone(),
      mode_options: mode_options.clone(),
    })?;
    println!("wrote {}", path.display());
  }

  if let Some(path) = cli.export_asciinema.as_ref() {
    let playlist = make_playlist(&name, favorite_filter)?;
    export::export(playlist, ExportConfig {
      format: ExportFormat::Asciinema,
      path: path.clone(),
      fps: cli.fps,
      seconds: export_seconds(cli.seconds),
      width: cli.width.unwrap_or(100),
      height: cli.height.unwrap_or(40),
      options: initial_options.clone(),
      mode_options: mode_options.clone(),
    })?;
    println!("wrote {}", path.display());
  }

  if cli.export_gif.is_some() || cli.export_asciinema.is_some() {
    return Ok(());
  }

  let (playlist, run_seconds) = make_interactive_playlist(&name, cli.seconds, favorite_filter)?;

  runner::run(playlist, RunConfig {
    fps: cli.fps,
    seconds: run_seconds,
    width: cli.width,
    height: cli.height,
    options: initial_options,
    mode_options,
    saved_mode_options,
    favorites,
    no_status: cli.no_status,
    settings_path: settings::DEFAULT_PATH.to_string(),
  })
}

fn make_playlist(name: &str, favorite_filter: Option<&[String]>) -> std::io::Result<Playlist> {
  let playlist = match name {
    "random" => match favorite_filter {
      Some(names) => Playlist::random_from(names).ok_or_else(no_favorites_error)?,
      None => Playlist::random(),
    },
    "cycle" => match favorite_filter {
      Some(names) => Playlist::cycle_from(names).ok_or_else(no_favorites_error)?,
      None => Playlist::cycle(),
    },
    other => {
      if registry::info(other).is_none() {
        eprintln!("Unknown mode: {}", other);
        eprintln!("Try --list to see available modes.");
        std::process::exit(2);
      }
      if let Some(names) = favorite_filter {
        if !contains_name(names, other) {
          eprintln!("Mode is not in favorites: {}", other);
          eprintln!("Run with --list --favorites to see saved favorites.");
          std::process::exit(2);
        }
        Playlist::single_from(other, names).ok_or_else(no_favorites_error)?
      } else {
        Playlist::single(other)
      }
    }
  };
  Ok(playlist)
}

fn make_interactive_playlist(name: &str, seconds: f64, favorite_filter: Option<&[String]>) -> std::io::Result<(Playlist, f64)> {
  let clip = if seconds > 0.0 { seconds } else { crate::playlist::CLIP_SECONDS };
  let result = match name {
    "random" => match favorite_filter {
      Some(names) => (Playlist::random_with_clip_from(clip, names).ok_or_else(no_favorites_error)?, 0.0),
      None => (Playlist::random_with_clip(clip), 0.0),
    },
    "cycle" => match favorite_filter {
      Some(names) => (Playlist::cycle_with_clip_from(clip, names).ok_or_else(no_favorites_error)?, 0.0),
      None => (Playlist::cycle_with_clip(clip), 0.0),
    },
    _ => (make_playlist(name, favorite_filter)?, 0.0),
  };
  Ok(result)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn single_interactive_modes_ignore_seconds() {
    let (_, run_seconds) = make_interactive_playlist("plasma", 9.0, None).unwrap();
    assert_eq!(run_seconds, 0.0);
  }
}

fn export_seconds(seconds: f64) -> f64 {
  if seconds > 0.0 { seconds } else { 5.0 }
}

fn print_mode_list(favorites_only: bool, favorites: &[String]) {
  if favorites_only {
    println!("Favorite modes:");
    let mut printed = false;
    for name in favorites {
      if let Some(info) = registry::info(name) {
        println!("  {:<16} {}", info.name, info.description);
        printed = true;
      }
    }
    if !printed {
      println!("  (none saved yet; press f while running a mode)");
    }
  } else {
    println!("Available modes:");
    for info in registry::MODES {
      println!("  {:<16} {}", info.name, info.description);
    }
  }
  println!();
  println!("Playlists:");
  if favorites_only {
    println!("  {:<16} shuffle through favorite modes", "random");
    println!("  {:<16} step through favorite modes in order", "cycle");
  } else {
    println!("  {:<16} shuffle through every mode", "random");
    println!("  {:<16} step through every mode in order", "cycle");
  }
}

fn known_favorites(saved: &[String]) -> Vec<String> {
  let mut out = Vec::new();
  for name in saved {
    if registry::info(name).is_some() && !contains_name(&out, name) {
      out.push(name.clone());
    }
  }
  out
}

fn contains_name(names: &[String], needle: &str) -> bool {
  names.iter().any(|name| name == needle)
}

fn apply_charset_support(mode: &str, options: &mut RenderOptions) {
  options.charset = normalize_charset(&options.charset);
  if !registry::supports_charset(mode) {
    options.charset = "scene".to_string();
  }
}

fn no_favorites_error() -> io::Error {
  io::Error::new(
    ErrorKind::InvalidInput,
    format!("no favorites saved in {}; press f while running a mode", settings::DEFAULT_PATH),
  )
}
