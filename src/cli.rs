//! Command-line surface and main entry. Matches the Python CLI flags.

use std::collections::BTreeMap;
use std::path::PathBuf;

use clap::Parser;

use crate::export::{self, ExportConfig, ExportFormat};
use crate::options::RenderOptions;
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
  #[arg(long)] pub width: Option<usize>,
  #[arg(long)] pub height: Option<usize>,
  #[arg(long, default_value_t = 24.0)] pub fps: f64,
  /// Runtime limit for single interactive modes; per-scene duration for interactive playlists; total clip length for exports.
  #[arg(long, default_value_t = 0.0)] pub seconds: f64,
  #[arg(long)] pub scale: Option<f64>,
  #[arg(long)] pub contrast: Option<f64>,
  #[arg(long)] pub brightness: Option<f64>,
  #[arg(long)] pub speed: Option<f64>,
  #[arg(long)] pub theme: Option<String>,
  #[arg(long)] pub no_status: bool,
  #[arg(long, default_value_t = false)] pub blocks: bool,
  #[arg(long, default_value = "clean")] pub charset: String,
  #[arg(long, default_value_t = false)] pub scroll: bool,

  /// Export an animated GIF clip instead of running interactively.
  #[arg(long, value_name = "PATH")] pub export_gif: Option<PathBuf>,

  /// Export an asciinema v2 .cast clip instead of running interactively.
  #[arg(long, value_name = "PATH")] pub export_asciinema: Option<PathBuf>,
}

pub fn run() -> std::io::Result<()> {
  let cli = Cli::parse();

  if cli.list {
    println!("Available modes:");
    for info in registry::MODES {
      println!("  {:<14} {}", info.name, info.description);
    }
    println!();
    println!("Playlists:");
    println!("  {:<14} shuffle through every mode", "random");
    println!("  {:<14} step through every mode in order", "cycle");
    return Ok(());
  }

  let name = cli.mode.clone().unwrap_or_else(|| "wave-plane".to_string());

  let saved = settings::load(settings::DEFAULT_PATH);

  let make_options = |base_name: &str| -> RenderOptions {
    let mut opt = RenderOptions::default();
    if let Some(entry) = saved.get(base_name) {
      entry.apply(&mut opt);
    }
    if let Some(v) = cli.scale { opt.scale = v; }
    if let Some(v) = cli.contrast { opt.contrast = v; }
    if let Some(v) = cli.brightness { opt.brightness = v; }
    if let Some(v) = cli.speed { opt.speed = v; }
    if let Some(ref v) = cli.theme { opt.theme = v.clone(); }
    opt.blocks = cli.blocks;
    opt.scroll = cli.scroll;
    opt.charset = cli.charset.clone();
    opt
  };

  let mode_options: BTreeMap<String, RenderOptions> = registry::MODES
    .iter()
    .map(|m| (m.name.to_string(), make_options(m.name)))
    .collect();

  let playlist = make_playlist(&name)?;
  let initial_options = mode_options
    .get(playlist.name())
    .cloned()
    .unwrap_or_else(|| make_options(playlist.name()));

  if let Some(path) = cli.export_gif.as_ref() {
    let playlist = make_playlist(&name)?;
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
    let playlist = make_playlist(&name)?;
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

  let (playlist, run_seconds) = make_interactive_playlist(&name, cli.seconds)?;

  runner::run(playlist, RunConfig {
    fps: cli.fps,
    seconds: run_seconds,
    width: cli.width,
    height: cli.height,
    options: initial_options,
    mode_options,
    no_status: cli.no_status,
    settings_path: settings::DEFAULT_PATH.to_string(),
  })
}

fn make_playlist(name: &str) -> std::io::Result<Playlist> {
  let playlist = match name {
    "random" => Playlist::random(),
    "cycle" => Playlist::cycle(),
    other => {
      if registry::create(other).is_none() {
        eprintln!("Unknown mode: {}", other);
        eprintln!("Try --list to see available modes.");
        std::process::exit(2);
      }
      Playlist::single(other)
    }
  };
  Ok(playlist)
}

fn make_interactive_playlist(name: &str, seconds: f64) -> std::io::Result<(Playlist, f64)> {
  let clip = if seconds > 0.0 { seconds } else { crate::playlist::CLIP_SECONDS };
  let result = match name {
    "random" => (Playlist::random_with_clip(clip), 0.0),
    "cycle" => (Playlist::cycle_with_clip(clip), 0.0),
    _ => (make_playlist(name)?, seconds),
  };
  Ok(result)
}

fn export_seconds(seconds: f64) -> f64 {
  if seconds > 0.0 { seconds } else { 5.0 }
}
