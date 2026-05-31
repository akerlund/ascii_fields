//! The `Animation` trait every mode implements.

use crate::options::RenderOptions;

pub struct FrameContext<'a> {
  pub width: usize,
  pub height: usize,
  pub elapsed: f64,
  pub phase: f64,
  pub options: &'a RenderOptions,
}

/// Each animation writes its ANSI-coded frame straight into `out`. `&mut self`
/// because some scenes carry per-frame state (life, dla, drops, …).
pub trait Animation: Send {
  fn render(&mut self, ctx: &FrameContext, out: &mut String);
}
