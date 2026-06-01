use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, render_field, FieldStyle};
use std::collections::HashMap;

const STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "nebula" };
const TH: &[(f64, char)] = &[
  (0.05, ' '),
  (0.14, '.'),
  (0.26, ':'),
  (0.40, '-'),
  (0.54, '='),
  (0.68, '+'),
  (0.80, '*'),
  (0.90, '#'),
  (1.01, '@'),
];

type OrbFn = fn(f64, f64, f64) -> f64;
// (label, function, world half-extent)
const ORBITALS: &[(&str, OrbFn, f64)] = &[
  ("1s", orb_1s, 6.0),
  ("2p", orb_2pz, 12.0),
  ("2s", orb_2s, 14.0),
  ("3d_z2", orb_3dz2, 20.0),
  ("3d_xz", orb_3dxz, 20.0),
  ("4f", orb_4fz3, 26.0),
];
const STATE_SECONDS: f64 = 4.5;
const TRANSITION: f64 = 1.6;

fn orb_1s(_x: f64, _z: f64, r: f64) -> f64 {
  (-r).exp()
}
fn orb_2s(_x: f64, _z: f64, r: f64) -> f64 {
  (1.0 - 0.5 * r) * (-0.5 * r).exp()
}
fn orb_2pz(_x: f64, z: f64, r: f64) -> f64 {
  z * (-0.5 * r).exp()
}
fn orb_3dz2(_x: f64, z: f64, r: f64) -> f64 {
  (3.0 * z * z - r * r) * (-r / 3.0).exp()
}
fn orb_3dxz(x: f64, z: f64, r: f64) -> f64 {
  x * z * (-r / 3.0).exp()
}
fn orb_4fz3(_x: f64, z: f64, r: f64) -> f64 {
  z * (5.0 * z * z - 3.0 * r * r) * (-0.25 * r).exp()
}

pub struct Orbitals {
  norm: HashMap<usize, f64>,
}
impl Default for Orbitals {
  fn default() -> Self {
    Self { norm: HashMap::new() }
  }
}

impl Orbitals {
  fn normalizer(&mut self, idx: usize) -> f64 {
    if let Some(&v) = self.norm.get(&idx) {
      return v;
    }
    let (_, f, ext) = ORBITALS[idx];
    let mut peak: f64 = 1e-9;
    for i in 0..=40 {
      for j in 0..=40 {
        let x = (i as f64 / 40.0 - 0.5) * 2.0 * ext;
        let z = (j as f64 / 40.0 - 0.5) * 2.0 * ext;
        let a = f(x, z, (x * x + z * z).sqrt());
        if a * a > peak {
          peak = a * a;
        }
      }
    }
    let n = 1.0 / peak;
    self.norm.insert(idx, n);
    n
  }
}

impl Animation for Orbitals {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let w = ctx.width;
    let h = ctx.height;
    let cycle = STATE_SECONDS;
    let pos = ctx.elapsed / cycle;
    let i0 = (pos as usize) % ORBITALS.len();
    let i1 = (i0 + 1) % ORBITALS.len();
    let frac = pos - pos.floor();
    let blend = clamp((frac - (1.0 - TRANSITION / cycle)) / (TRANSITION / cycle));
    let (_, f0, ext0) = ORBITALS[i0];
    let (_, f1, ext1) = ORBITALS[i1];
    let n0 = self.normalizer(i0);
    let n1 = self.normalizer(i1);
    let contrast = ctx.options.contrast;
    let mut grid = vec![0.0_f64; w * h];
    for row in 0..h {
      let pz = (row as f64 - (h as f64 - 1.0) * 0.5) / (h as f64 * 0.5).max(1.0);
      let base = row * w;
      for col in 0..w {
        let px = (col as f64 - (w as f64 - 1.0) * 0.5) / (w as f64 * 0.5).max(1.0);
        let x0 = px * ext0;
        let z0 = pz * ext0;
        let a0 = f0(x0, z0, (x0 * x0 + z0 * z0).sqrt());
        let d0 = a0 * a0 * n0;
        let d = if blend > 0.0 {
          let x1 = px * ext1;
          let z1 = pz * ext1;
          let a1 = f1(x1, z1, (x1 * x1 + z1 * z1).sqrt());
          d0 * (1.0 - blend) + a1 * a1 * n1 * blend
        } else {
          d0
        };
        let level = clamp(d).powf(0.45);
        grid[base + col] = clamp(level * contrast);
      }
    }
    render_field(ctx, &grid, TH, &STYLE, out);
  }
}
