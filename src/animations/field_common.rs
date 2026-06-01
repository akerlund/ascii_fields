use crate::animation::FrameContext;

const PULSE_MIN_WIDTH: f64 = 1e-5;

pub(super) const FIELD_TH: &[(f64, char)] = &[
  (0.08, ' '), (0.18, '.'), (0.30, ':'), (0.43, '-'), (0.56, '='),
  (0.69, '+'), (0.81, '*'), (0.92, '#'), (1.01, '@'),
];

pub(super) const LINE_TH: &[(f64, char)] = &[
  (0.08, ' '), (0.20, '.'), (0.36, ':'), (0.52, '-'), (0.68, '='),
  (0.80, '+'), (0.90, '*'), (0.97, '#'), (1.01, '@'),
];

#[inline]
pub(super) fn dims(ctx: &FrameContext) -> (usize, usize, f64, f64) {
  let w = ctx.width;
  let h = ctx.height;
  let dw = w.saturating_sub(1).max(1) as f64;
  let dh = h.saturating_sub(1).max(1) as f64;
  (w, h, dw, dh)
}

#[inline]
pub(super) fn aspect(ctx: &FrameContext) -> f64 {
  ctx.width as f64 / (ctx.height as f64 * 2.0).max(1.0)
}

#[inline]
pub(super) fn pulse(x: f64, width: f64) -> f64 {
  (-(x * x) / width.max(PULSE_MIN_WIDTH)).exp()
}

#[derive(Default)]
pub(super) struct FrameScratch {
  grid: Vec<f64>,
  glyphs: Vec<char>,
}

impl FrameScratch {
  #[inline]
  pub(super) fn grid(&mut self, len: usize) -> &mut [f64] {
    self.grid.resize(len, 0.0);
    self.grid.fill(0.0);
    self.grid.as_mut_slice()
  }

  #[inline]
  pub(super) fn grid_and_glyphs(&mut self, len: usize) -> (&mut [f64], &mut [char]) {
    self.grid.resize(len, 0.0);
    self.glyphs.resize(len, ' ');
    self.grid.fill(0.0);
    self.glyphs.fill(' ');
    (self.grid.as_mut_slice(), self.glyphs.as_mut_slice())
  }
}

#[inline]
pub(super) fn put(grid: &mut [f64], glyphs: &mut [char], w: usize, h: usize, col: i64, row: i64, level: f64, ch: char) {
  if col >= 0 && (col as usize) < w && row >= 0 && (row as usize) < h {
    let idx = row as usize * w + col as usize;
    if level > grid[idx] {
      grid[idx] = level;
      glyphs[idx] = ch;
    }
  }
}
