//! Persist per-mode RenderOptions and favorites to a single JSON file.

use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::options::RenderOptions;

pub const DEFAULT_PATH: &str = "ascii_fields.json";

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
  #[serde(default)] pub theme: Option<String>,
  #[serde(default)] pub scale: Option<f64>,
  #[serde(default)] pub contrast: Option<f64>,
  #[serde(default)] pub brightness: Option<f64>,
  #[serde(default)] pub speed: Option<f64>,
}

impl SavedMode {
  pub fn from(options: &RenderOptions) -> Self {
    Self {
      theme: Some(options.theme.clone()),
      scale: Some(options.scale),
      contrast: Some(options.contrast),
      brightness: Some(options.brightness),
      speed: Some(options.speed),
    }
  }

  pub fn apply(&self, options: &mut RenderOptions) {
    if let Some(ref v) = self.theme { options.theme = v.clone(); }
    if let Some(v) = self.scale { options.scale = v; }
    if let Some(v) = self.contrast { options.contrast = v; }
    if let Some(v) = self.brightness { options.brightness = v; }
    if let Some(v) = self.speed { options.speed = v; }
  }
}

pub fn load(path: &str) -> SavedConfig {
  if !Path::new(path).exists() { return SavedConfig::default(); }
  let data = match std::fs::read_to_string(path) {
    Ok(s) => s,
    Err(_) => return SavedConfig::default(),
  };
  parse(&data)
}

pub fn save(path: &str, options_by_mode: &BTreeMap<String, RenderOptions>, favorites: &[String]) -> std::io::Result<()> {
  let modes: BTreeMap<String, SavedMode> = options_by_mode
    .iter()
    .map(|(k, v)| (k.clone(), SavedMode::from(v)))
    .collect();
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
    let modes = value
      .get("modes")
      .and_then(|v| serde_json::from_value(v.clone()).ok())
      .unwrap_or_default();
    let favorites = value
      .get("favorites")
      .and_then(|v| serde_json::from_value(v.clone()).ok())
      .unwrap_or_default();
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
    let saved = parse(r#"{"plasma":{"theme":"scene","scale":1.2}}"#);
    assert!(saved.favorites.is_empty());
    assert_eq!(saved.modes["plasma"].theme.as_deref(), Some("scene"));
    assert_eq!(saved.modes["plasma"].scale, Some(1.2));
  }

  #[test]
  fn reads_structured_file_with_deduped_favorites() {
    let saved = parse(r#"{
      "favorites": ["plasma", "caustics", "plasma", ""],
      "modes": {"caustics":{"theme":"bathymetry"}}
    }"#);
    assert_eq!(saved.favorites, vec!["plasma", "caustics"]);
    assert_eq!(saved.modes["caustics"].theme.as_deref(), Some("bathymetry"));
  }
}
