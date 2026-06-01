use crate::animation::{Animation, FrameContext};
use crate::core::{render_field, FieldStyle};
use std::f64::consts::PI;

const STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "nebula" };
const TH: &[(f64, char)] =
  &[(0.06, ' '), (0.26, '.'), (0.42, ':'), (0.56, '-'), (0.68, '+'), (0.80, '*'), (0.90, '#'), (1.01, '@')];
const N_ATOMS: usize = 42;

fn fibonacci_sphere(n: usize) -> Vec<(f64, f64, f64)> {
  let mut v = Vec::with_capacity(n);
  let ga = PI * (3.0 - 5.0_f64.sqrt());
  let denom = (n - 1).max(1) as f64;
  for i in 0..n {
    let y = 1.0 - (i as f64 / denom) * 2.0;
    let r = (1.0 - y * y).max(0.0).sqrt();
    let theta = ga * i as f64;
    v.push((theta.cos() * r, y, theta.sin() * r));
  }
  v
}

fn build_bonds(atoms: &[(f64, f64, f64)]) -> Vec<(usize, usize)> {
  let mut bonds = Vec::new();
  let n = atoms.len();
  for i in 0..n {
    let mut dists: Vec<(usize, f64)> = (0..n)
      .map(|j| {
        let d = (atoms[i].0 - atoms[j].0).powi(2)
          + (atoms[i].1 - atoms[j].1).powi(2)
          + (atoms[i].2 - atoms[j].2).powi(2);
        (j, d)
      })
      .collect();
    dists.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    for &(j, _) in &dists[1..4] {
      let edge = (i.min(j), i.max(j));
      if !bonds.contains(&edge) {
        bonds.push(edge);
      }
    }
  }
  bonds
}

pub struct Molecule {
  atoms: Vec<(f64, f64, f64)>,
  bonds: Vec<(usize, usize)>,
}
impl Default for Molecule {
  fn default() -> Self {
    let atoms = fibonacci_sphere(N_ATOMS);
    let bonds = build_bonds(&atoms);
    Self { atoms, bonds }
  }
}

impl Animation for Molecule {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let w = ctx.width;
    let h = ctx.height;
    let mut grid = vec![0.0_f64; w * h];
    let ay = ctx.elapsed * 0.6;
    let ax = ctx.elapsed * 0.37;
    let (cay, say) = (ay.cos(), ay.sin());
    let (cax, sax) = (ax.cos(), ax.sin());
    let gain = w.min(h * 2) as f64 * 0.55;
    let cx = w as f64 * 0.5;
    let cy = h as f64 * 0.5;
    let mut pts: Vec<(f64, f64, f64)> = Vec::with_capacity(self.atoms.len());
    for &(x, y, z) in &self.atoms {
      let (x, z) = (x * cay - z * say, x * say + z * cay);
      let (y, z) = (y * cax - z * sax, y * sax + z * cax);
      let k = 1.0 / (2.6 - z);
      pts.push((cx + x * k * gain, cy + y * k * gain * 0.5, z));
    }
    let put = |grid: &mut Vec<f64>, col: f64, row: f64, val: f64| {
      let c = col.round() as i64;
      let r = row.round() as i64;
      if c >= 0 && (c as usize) < w && r >= 0 && (r as usize) < h {
        let idx = (r as usize) * w + (c as usize);
        if grid[idx] < val {
          grid[idx] = val;
        }
      }
    };
    for &(i, j) in &self.bonds {
      let (x0, y0, z0) = pts[i];
      let (x1, y1, z1) = pts[j];
      let steps = ((x1 - x0).hypot(y1 - y0).round() as i64).max(1);
      for s in 0..=steps {
        let f = s as f64 / steps as f64;
        let z = z0 + (z1 - z0) * f;
        put(&mut grid, x0 + (x1 - x0) * f, y0 + (y1 - y0) * f, 0.30 + 0.30 * (z * 0.5 + 0.5));
      }
    }
    for &(x, y, z) in &pts {
      let bright = 0.55 + 0.45 * (z * 0.5 + 0.5);
      put(&mut grid, x, y, bright);
      put(&mut grid, x + 1.0, y, bright * 0.8);
      put(&mut grid, x, y + 1.0, bright * 0.7);
    }
    render_field(ctx, &grid, TH, &STYLE, out);
  }
}
