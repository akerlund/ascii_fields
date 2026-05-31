//! Per-mode render knobs. Mutable so the runner can tune them live from keys.

#[derive(Clone, Debug)]
pub struct RenderOptions {
  pub scale: f64,
  pub contrast: f64,
  pub brightness: f64,
  pub speed: f64,
  /// Theme name. `"grayscale"` keeps the classic mono look, `"scene"` defers
  /// to each animation's `default_theme`, anything else forces that palette.
  pub theme: String,
  /// Wave-plane character ramp ("clean" | "soft" | "dense").
  pub charset: String,
  pub scroll: bool,
  pub blocks: bool,
}

impl Default for RenderOptions {
  fn default() -> Self {
    Self {
      scale: 1.0,
      contrast: 1.05,
      brightness: 1.0,
      speed: 1.0,
      theme: "grayscale".into(),
      charset: "clean".into(),
      scroll: false,
      blocks: false,
    }
  }
}
