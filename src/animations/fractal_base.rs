//! Shared escape-time helpers for the fractal scenes (mandel / julia /
//! burning-ship / newton). Mandelbrot zoom is sawtooth-continuous; the others
//! use a soft oscillation.

use std::f64::consts::LN_2;

pub fn axes(width: usize, height: usize, cx: f64, cy: f64, scale: f64) -> (Vec<f64>, Vec<f64>) {
  let aspect = width as f64 / (height as f64 * 2.0).max(1.0);
  let dw = (width.saturating_sub(1)).max(1) as f64;
  let dh = (height.saturating_sub(1)).max(1) as f64;
  let xs = (0..width).map(|c| cx + (c as f64 / dw - 0.5) * 2.0 * scale * aspect).collect();
  let ys = (0..height).map(|r| cy + (r as f64 / dh - 0.5) * 2.0 * scale).collect();
  (xs, ys)
}

#[inline]
pub fn smooth(i: usize, zr2: f64, zi2: f64, max_iter: usize) -> f64 {
  let mag2 = zr2 + zi2;
  if mag2 <= 1.0 { return i as f64 / max_iter as f64; }
  let nu = i as f64 + 1.0 - (0.5 * mag2.ln()).ln() / LN_2;
  (nu / max_iter as f64).clamp(0.0, 1.0)
}

pub fn iteration_cap(width: usize, height: usize, lo: usize, hi: usize) -> usize {
  let cells = (width * height).max(1);
  ((700_000 / cells) + 40).clamp(lo, hi)
}

pub fn mandelbrot_cell(cr: f64, ci: f64, max_iter: usize, ship: bool) -> f64 {
  let mut zr = 0.0_f64; let mut zi = 0.0_f64;
  for i in 0..max_iter {
    let (mut sr, mut si) = (zr, zi);
    if ship { sr = sr.abs(); si = si.abs(); }
    let zr2 = sr * sr; let zi2 = si * si;
    if zr2 + zi2 > 16.0 { return smooth(i, zr2, zi2, max_iter); }
    let nzi = 2.0 * sr * si + ci;
    let nzr = zr2 - zi2 + cr;
    zr = nzr; zi = nzi;
  }
  0.0
}

pub fn julia_cell(mut zr: f64, mut zi: f64, cr: f64, ci: f64, max_iter: usize) -> f64 {
  for i in 0..max_iter {
    let zr2 = zr * zr; let zi2 = zi * zi;
    if zr2 + zi2 > 16.0 { return smooth(i, zr2, zi2, max_iter); }
    let nzi = 2.0 * zr * zi + ci;
    let nzr = zr2 - zi2 + cr;
    zr = nzr; zi = nzi;
  }
  0.0
}
