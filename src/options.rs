//! Per-mode render knobs. Mutable so the runner can tune them live from keys.

pub const CHARSET_CYCLE: &[&str] =
  &["scene", "clean", "soft", "dense", "minimal", "smooth", "sharp", "matrix", "braille", "blocks"];

pub fn normalize_charset(name: &str) -> String {
  if CHARSET_CYCLE.iter().any(|charset| *charset == name) {
    name.to_string()
  } else {
    "scene".to_string()
  }
}

#[derive(Clone, Debug)]
pub struct RenderOptions {
  pub scale: f64,
  pub contrast: f64,
  pub brightness: f64,
  pub speed: f64,
  /// Theme name. `"grayscale"` keeps the classic mono look, `"scene"` defers
  /// to each animation's `default_theme`, anything else forces that palette.
  pub theme: String,
  /// Density-field rendering mode. One of `CHARSET_CYCLE`. "scene" defers to
  /// each animation's preferred thresholds; "blocks" switches to the bg-cell
  /// renderer; the others select a foreground glyph ramp.
  pub charset: String,
  pub scroll: bool,
}

impl Default for RenderOptions {
  fn default() -> Self {
    Self {
      scale: 1.0,
      contrast: 1.05,
      brightness: 1.0,
      speed: 1.0,
      theme: "grayscale".into(),
      charset: "scene".into(),
      scroll: false,
    }
  }
}
