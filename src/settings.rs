//! Persist per-mode RenderOptions to a single JSON file.

use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::options::RenderOptions;

pub const DEFAULT_PATH: &str = "ascii_fields.json";

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

pub fn load(path: &str) -> BTreeMap<String, SavedMode> {
  if !Path::new(path).exists() { return BTreeMap::new(); }
  let data = match std::fs::read_to_string(path) {
    Ok(s) => s,
    Err(_) => return BTreeMap::new(),
  };
  serde_json::from_str(&data).unwrap_or_default()
}

pub fn save(path: &str, options_by_mode: &BTreeMap<String, RenderOptions>) -> std::io::Result<()> {
  let data: BTreeMap<&String, SavedMode> = options_by_mode
    .iter()
    .map(|(k, v)| (k, SavedMode::from(v)))
    .collect();
  let json = serde_json::to_string_pretty(&data).unwrap_or_default();
  std::fs::write(path, json + "\n")
}
