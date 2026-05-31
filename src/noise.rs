//! Deterministic value noise + fractional Brownian motion. Matches the
//! Python `noise.py` API so animations port over directly.

#[inline]
pub fn hash01(ix: i64, iy: i64, seed: i64) -> f64 {
  let mut v = (ix.wrapping_mul(374761393) + iy.wrapping_mul(668265263)
               + seed.wrapping_mul(1442695040888963407)) as u64;
  v &= 0xFFFFFFFF;
  v = (v ^ (v >> 13)).wrapping_mul(1274126177);
  v &= 0xFFFFFFFF;
  v ^= v >> 16;
  (v & 0xFFFF) as f64 / 65535.0
}

#[inline]
pub fn value_noise(x: f64, y: f64, seed: i64) -> f64 {
  let ix = x.floor() as i64;
  let iy = y.floor() as i64;
  let fx = x - ix as f64;
  let fy = y - iy as f64;
  let ux = fx * fx * (3.0 - 2.0 * fx);
  let uy = fy * fy * (3.0 - 2.0 * fy);
  let a = hash01(ix, iy, seed);
  let b = hash01(ix + 1, iy, seed);
  let c = hash01(ix, iy + 1, seed);
  let d = hash01(ix + 1, iy + 1, seed);
  let top = a + (b - a) * ux;
  let bot = c + (d - c) * ux;
  top + (bot - top) * uy
}

pub fn fbm(x: f64, y: f64, octaves: u32) -> f64 {
  fbm_seeded(x, y, octaves, 0)
}

pub fn fbm_seeded(x: f64, y: f64, octaves: u32, seed: i64) -> f64 {
  let mut amp = 0.5;
  let mut freq = 1.0;
  let mut total = 0.0;
  let mut norm = 0.0;
  for o in 0..octaves {
    total += amp * value_noise(x * freq, y * freq, seed + o as i64 * 101);
    norm += amp;
    amp *= 0.5;
    freq *= 2.0;
  }
  if norm > 0.0 { total / norm } else { 0.0 }
}

/// Cheap 2D star hash matching Python's `star_noise`.
#[inline]
pub fn star_noise(ix: i64, iy: i64) -> f64 {
  let mut v = (ix.wrapping_mul(374761393) + iy.wrapping_mul(668265263)) as u64;
  v &= 0xFFFFFFFF;
  v = (v ^ (v >> 13)).wrapping_mul(1274126177);
  v ^= v >> 16;
  (v & 0xFFFF) as f64 / 65535.0
}
