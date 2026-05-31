use crate::animation::{Animation, FrameContext};
use crate::core::{render_field, FieldStyle};

const STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "ice" };
const TH: &[(f64, char)] = &[
  (0.06, ' '), (0.30, '.'), (0.45, ':'), (0.58, '-'), (0.70, '+'),
  (0.82, '*'), (0.92, '#'), (1.01, '@'),
];

fn vertices() -> Vec<(f64, f64, f64, f64)> {
  let mut v = Vec::with_capacity(16);
  for w in &[-1.0, 1.0] {
    for z in &[-1.0, 1.0] {
      for y in &[-1.0, 1.0] {
        for x in &[-1.0, 1.0] { v.push((*x, *y, *z, *w)); }
      }
    }
  }
  v
}

fn edges() -> Vec<(usize, usize)> {
  let verts = vertices();
  let mut e = Vec::new();
  for i in 0..verts.len() {
    for j in (i + 1)..verts.len() {
      let a = verts[i]; let b = verts[j];
      let mut diff = 0;
      if a.0 != b.0 { diff += 1; }
      if a.1 != b.1 { diff += 1; }
      if a.2 != b.2 { diff += 1; }
      if a.3 != b.3 { diff += 1; }
      if diff == 1 { e.push((i, j)); }
    }
  }
  e
}

fn rotate(p: (f64, f64, f64, f64), axw: f64, ayz: f64, axy: f64) -> (f64, f64, f64, f64) {
  let (mut x, mut y, mut z, mut w) = p;
  let (c, s) = (axw.cos(), axw.sin()); let (nx, nw) = (x * c - w * s, x * s + w * c); x = nx; w = nw;
  let (c, s) = (ayz.cos(), ayz.sin()); let (ny, nz) = (y * c - z * s, y * s + z * c); y = ny; z = nz;
  let (c, s) = (axy.cos(), axy.sin()); let (nx, ny) = (x * c - y * s, x * s + y * c); x = nx; y = ny;
  (x, y, z, w)
}

pub struct Hypercube { edges: Vec<(usize, usize)>, verts: Vec<(f64, f64, f64, f64)> }
impl Default for Hypercube { fn default() -> Self { Self { edges: edges(), verts: vertices() } } }

impl Animation for Hypercube {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let w = ctx.width; let h = ctx.height;
    let mut grid = vec![0.0_f64; w * h];
    let axw = ctx.elapsed * 0.55;
    let ayz = ctx.elapsed * 0.37;
    let axy = ctx.elapsed * 0.20;
    let gain = (w.min(h * 2) as f64) * 0.62;
    let cx = w as f64 * 0.5; let cy = h as f64 * 0.5;
    let mut pts: Vec<(f64, f64)> = Vec::with_capacity(self.verts.len());
    let mut depths: Vec<f64> = Vec::with_capacity(self.verts.len());
    for &v in &self.verts {
      let (x, y, z, ww) = rotate(v, axw, ayz, axy);
      let k4 = 1.0 / (2.3 - ww);
      let (x, y, z) = (x * k4, y * k4, z * k4);
      let k3 = 1.0 / (2.4 - z);
      pts.push((cx + x * k3 * gain, cy + y * k3 * gain * 0.5));
      depths.push(k3);
    }
    let dmin = depths.iter().cloned().fold(f64::INFINITY, f64::min);
    let dmax = depths.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let span = (dmax - dmin).max(1.0);
    let put = |grid: &mut Vec<f64>, col: i64, row: i64, val: f64| {
      if col >= 0 && (col as usize) < w && row >= 0 && (row as usize) < h {
        let idx = (row as usize) * w + (col as usize);
        if grid[idx] < val { grid[idx] = val; }
      }
    };
    for &(i, j) in &self.edges {
      let (x0, y0) = pts[i]; let (x1, y1) = pts[j];
      let b0 = 0.35 + 0.65 * (depths[i] - dmin) / span;
      let b1 = 0.35 + 0.65 * (depths[j] - dmin) / span;
      let steps = ((x1 - x0).hypot(y1 - y0).round() as i64).max(1);
      for s in 0..=steps {
        let f = s as f64 / steps as f64;
        let px = (x0 + (x1 - x0) * f).round() as i64;
        let py = (y0 + (y1 - y0) * f).round() as i64;
        put(&mut grid, px, py, b0 + (b1 - b0) * f);
      }
    }
    render_field(ctx, &grid, TH, &STYLE, out);
  }
}
