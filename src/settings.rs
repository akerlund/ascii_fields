//! Persist per-mode RenderOptions and favorites to a single JSON file.
//!
//! The canonical location follows the XDG Base Directory spec:
//!   $XDG_CONFIG_HOME/ascii-fields/ascii_fields.json
//!   $HOME/.config/ascii-fields/ascii_fields.json   (fallback)
//!   ./ascii_fields.json                            (last-resort fallback)
//!
//! On first load, if no file exists at the canonical path but the legacy
//! `./ascii_fields.json` does, it is read so existing users do not lose their
//! settings. The next `s` or `f` keypress writes to the canonical path; the
//! user can then delete the legacy file at their leisure.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::options::{normalize_charset, RenderOptions};

pub const SETTINGS_FILE: &str = "ascii_fields.json";

/// Resolve the canonical settings path for the current user.
pub fn default_path() -> PathBuf {
  if let Some(dir) = config_dir() {
    return dir.join("ascii-fields").join(SETTINGS_FILE);
  }
  PathBuf::from(SETTINGS_FILE)
}

fn config_dir() -> Option<PathBuf> {
  if let Some(v) = nonempty_env("XDG_CONFIG_HOME") {
    return Some(PathBuf::from(v));
  }
  nonempty_env("HOME").map(|h| PathBuf::from(h).join(".config"))
}

fn nonempty_env(name: &str) -> Option<String> {
  std::env::var(name).ok().filter(|v| !v.is_empty())
}

fn legacy_path() -> PathBuf {
  PathBuf::from(SETTINGS_FILE)
}

#[derive(Clone, Debug, Default)]
pub struct SavedConfig {
  pub favorites: Vec<String>,
  pub modes: BTreeMap<String, SavedMode>,
}

#[derive(Debug, Serialize)]
struct SavedFile {
  favorites: Vec<String>,
  modes: BTreeMap<String, SavedMode>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SavedMode {
  #[serde(default)]
  pub theme: Option<String>,
  #[serde(default)]
  pub scale: Option<f64>,
  #[serde(default)]
  pub contrast: Option<f64>,
  #[serde(default)]
  pub brightness: Option<f64>,
  #[serde(default)]
  pub speed: Option<f64>,
  #[serde(default)]
  pub charset: Option<String>,
}

impl SavedMode {
  pub fn from(options: &RenderOptions) -> Self {
    Self {
      theme: Some(options.theme.clone()),
      scale: Some(options.scale),
      contrast: Some(options.contrast),
      brightness: Some(options.brightness),
      speed: Some(options.speed),
      charset: Some(normalize_charset(&options.charset)),
    }
  }

  pub fn apply(&self, options: &mut RenderOptions) {
    if let Some(ref v) = self.theme {
      options.theme = v.clone();
    }
    if let Some(v) = self.scale {
      options.scale = v;
    }
    if let Some(v) = self.contrast {
      options.contrast = v;
    }
    if let Some(v) = self.brightness {
      options.brightness = v;
    }
    if let Some(v) = self.speed {
      options.speed = v;
    }
    if let Some(ref v) = self.charset {
      options.charset = normalize_charset(v);
    }
  }
}

pub fn load(path: &Path) -> SavedConfig {
  if path.exists() {
    return read_file(path);
  }
  // First-run migration: pick up the pre-XDG ./ascii_fields.json if present.
  let legacy = legacy_path();
  if legacy != path && legacy.exists() {
    eprintln!(
      "ascii-fields: reading legacy settings from {}; press `s` or `f` to migrate to {}",
      legacy.display(),
      path.display()
    );
    return read_file(&legacy);
  }
  SavedConfig::default()
}

fn read_file(path: &Path) -> SavedConfig {
  match std::fs::read_to_string(path) {
    Ok(s) => parse(&s),
    Err(_) => SavedConfig::default(),
  }
}

pub fn save(
  path: &Path,
  options_by_mode: &BTreeMap<String, RenderOptions>,
  favorites: &[String],
) -> std::io::Result<()> {
  if let Some(parent) = path.parent() {
    if !parent.as_os_str().is_empty() {
      std::fs::create_dir_all(parent)?;
    }
  }
  let modes: BTreeMap<String, SavedMode> =
    options_by_mode.iter().map(|(k, v)| (k.clone(), SavedMode::from(v))).collect();
  let data = SavedFile { favorites: normalize_favorites(favorites.to_vec()), modes };
  let json = serde_json::to_string_pretty(&data)
    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
  std::fs::write(path, json + "\n")
}

fn parse(data: &str) -> SavedConfig {
  let value: Value = match serde_json::from_str(data) {
    Ok(v) => v,
    Err(_) => return SavedConfig::default(),
  };

  if value.get("modes").is_some() || value.get("favorites").is_some() {
    let modes = value.get("modes").and_then(|v| serde_json::from_value(v.clone()).ok()).unwrap_or_default();
    let favorites =
      value.get("favorites").and_then(|v| serde_json::from_value(v.clone()).ok()).unwrap_or_default();
    return SavedConfig { favorites: normalize_favorites(favorites), modes };
  }

  let modes = serde_json::from_value(value).unwrap_or_default();
  SavedConfig { favorites: Vec::new(), modes }
}

fn normalize_favorites(favorites: Vec<String>) -> Vec<String> {
  let mut out = Vec::new();
  for name in favorites {
    let name = name.trim();
    if name.is_empty() || out.iter().any(|saved| saved == name) {
      continue;
    }
    out.push(name.to_string());
  }
  out
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn reads_legacy_mode_map() {
    let saved = parse(r#"{"plasma":{"theme":"scene","scale":1.2,"charset":"dense"}}"#);
    assert!(saved.favorites.is_empty());
    assert_eq!(saved.modes["plasma"].theme.as_deref(), Some("scene"));
    assert_eq!(saved.modes["plasma"].scale, Some(1.2));
    assert_eq!(saved.modes["plasma"].charset.as_deref(), Some("dense"));
  }

  #[test]
  fn reads_structured_file_with_deduped_favorites() {
    let saved = parse(
      r#"{
      "favorites": ["plasma", "caustics", "plasma", ""],
      "modes": {"caustics":{"theme":"bathymetry","charset":"blocks"}}
    }"#,
    );
    assert_eq!(saved.favorites, vec!["plasma", "caustics"]);
    assert_eq!(saved.modes["caustics"].theme.as_deref(), Some("bathymetry"));
    assert_eq!(saved.modes["caustics"].charset.as_deref(), Some("blocks"));
  }

  #[test]
  fn saved_mode_round_trips_charset() {
    let mut options = RenderOptions::default();
    options.charset = "blocks".to_string();
    let saved = SavedMode::from(&options);

    let mut applied = RenderOptions::default();
    saved.apply(&mut applied);

    assert_eq!(saved.charset.as_deref(), Some("blocks"));
    assert_eq!(applied.charset, "blocks");
  }

  #[test]
  fn unknown_charset_falls_back_to_scene_when_applied() {
    let saved = SavedMode { charset: Some("sparkles".to_string()), ..SavedMode::default() };
    let mut applied = RenderOptions::default();
    applied.charset = "dense".to_string();

    saved.apply(&mut applied);

    assert_eq!(applied.charset, "scene");
  }

  // The env-var-driven tests run with a global mutex to avoid races between
  // each other; `cargo test` parallelises modules by default.
  use std::sync::Mutex;
  static ENV_GUARD: Mutex<()> = Mutex::new(());

  fn with_env<F: FnOnce()>(xdg: Option<&str>, home: Option<&str>, body: F) {
    let _g = ENV_GUARD.lock().unwrap();
    let prev_xdg = std::env::var("XDG_CONFIG_HOME").ok();
    let prev_home = std::env::var("HOME").ok();
    match xdg {
      Some(v) => std::env::set_var("XDG_CONFIG_HOME", v),
      None => std::env::remove_var("XDG_CONFIG_HOME"),
    }
    match home {
      Some(v) => std::env::set_var("HOME", v),
      None => std::env::remove_var("HOME"),
    }
    body();
    match prev_xdg {
      Some(v) => std::env::set_var("XDG_CONFIG_HOME", v),
      None => std::env::remove_var("XDG_CONFIG_HOME"),
    }
    match prev_home {
      Some(v) => std::env::set_var("HOME", v),
      None => std::env::remove_var("HOME"),
    }
  }

  #[test]
  fn default_path_prefers_xdg_config_home() {
    with_env(Some("/x/config"), Some("/h"), || {
      assert_eq!(default_path(), PathBuf::from("/x/config/ascii-fields/ascii_fields.json"));
    });
  }

  #[test]
  fn default_path_falls_back_to_home_dot_config() {
    with_env(None, Some("/h"), || {
      assert_eq!(default_path(), PathBuf::from("/h/.config/ascii-fields/ascii_fields.json"));
    });
  }

  #[test]
  fn default_path_last_resort_is_cwd() {
    with_env(Some(""), Some(""), || {
      assert_eq!(default_path(), PathBuf::from("ascii_fields.json"));
    });
  }
}
