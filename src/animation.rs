//! The `Animation` trait every mode implements.

use crate::options::RenderOptions;

pub const THEME_COLOR_STEPS: usize = 8;

pub struct FrameContext<'a> {
  pub width: usize,
  pub height: usize,
  pub elapsed: f64,
  pub phase: f64,
  pub color_steps: usize,
  pub options: &'a RenderOptions,
}

/// Each animation writes its ANSI-coded frame straight into `out`. `&mut self`
/// because some scenes carry per-frame state (life, dla, drops, …).
pub trait Animation: Send {
  fn render(&mut self, ctx: &FrameContext, out: &mut String);

  /// One-line mode-specific status the HUD shows on tall terminals.
  /// Animations override this to expose live state (blob count for
  /// vax_lamp, particle count for feynman, generation for life, etc.).
  /// Default empty so most modes do not need to opt in.
  fn status(&self) -> Option<String> {
    None
  }
}
