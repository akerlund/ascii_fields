//! Apollonian gasket — the classical limit set of the Apollonian group of
//! mutually tangent circles. Acts as our stand-in for a "cohomology
//! fractal", since the Apollonian limit set arises as the boundary of the
//! universal cover of a triply-punctured sphere and its filigree of nested
//! circles is the same family of objects cohomology-fractal renderings of
//! hyperbolic 3-manifolds produce.
//!
//! Construction: start with three mutually tangent unit circles plus the
//! outer enclosing circle (Descartes configuration). At each step, every
//! triple of mutually tangent circles defines a fourth tangent circle via
//! Descartes' Circle Theorem; add it to the active set, recurse. We
//! generate a fixed-depth gasket once per frame (cheap because each step
//! doubles the count but we cap at ~3000 circles) and render every circle
//! as a soft ring of brightness.
//!
//! Slow rotation around the gasket's centre and a breathing zoom give the
//! "infinite zoom" feeling without actually needing depth refinement; the
//! gasket's self-similarity does the work.

use std::f64::consts::TAU;

use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, render_field, FieldStyle};

const STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "copper" };
const TH: &[(f64, char)] = &[
  (0.06, ' '),
  (0.14, '.'),
  (0.24, ':'),
  (0.36, '-'),
  (0.50, '='),
  (0.64, '+'),
  (0.78, '*'),
  (0.90, '#'),
  (1.01, '@'),
];

const MAX_CIRCLES: usize = 3000;
/// Stop subdividing once a child circle's radius drops below this fraction
/// of the enclosing circle. Keeps the recursion finite.
const MIN_RADIUS: f64 = 0.004;

#[derive(Clone, Copy)]
struct Circle {
  /// Curvature (1/r). Signed: outer enclosing circle gets a negative
  /// curvature so Descartes' theorem signs work out.
  k: f64,
  x: f64,
  y: f64,
}

impl Circle {
  #[inline]
  fn radius(self) -> f64 {
    1.0 / self.k.abs()
  }
}

/// Descartes' Circle Theorem: given three mutually tangent circles with
/// signed curvatures k1, k2, k3, the curvature of a fourth tangent circle is
///   k4 = k1 + k2 + k3 ± 2·sqrt(k1·k2 + k2·k3 + k3·k1)
/// The two signs give the two possible new circles. If one of those is
/// already present in the gasket, the other is the "next" curvature you
/// want.
///
/// Complex form (Descartes-Apollonius):
///   k4·z4 = k1·z1 + k2·z2 + k3·z3 ± 2·sqrt(k1·z1·k2·z2 + ... )
/// with the sign chosen consistent with the curvature equation.
fn fourth_circle(c1: Circle, c2: Circle, c3: Circle, sign: f64) -> Option<Circle> {
  let k_sum = c1.k + c2.k + c3.k;
  let cross = c1.k * c2.k + c2.k * c3.k + c3.k * c1.k;
  if cross < 0.0 {
    return None;
  }
  let k4 = k_sum + sign * 2.0 * cross.sqrt();
  // Complex form for centre: re-using the same combination on (kx, ky).
  let kxsum = c1.k * c1.x + c2.k * c2.x + c3.k * c3.x;
  let kysum = c1.k * c1.y + c2.k * c2.y + c3.k * c3.y;
  // Inner product in the complex sense for the sqrt term
  let cross_x = c1.k * c2.k * (c1.x * c2.x - c1.y * c2.y)
    + c2.k * c3.k * (c2.x * c3.x - c2.y * c3.y)
    + c3.k * c1.k * (c3.x * c1.x - c3.y * c1.y);
  let cross_y = c1.k * c2.k * (c1.x * c2.y + c1.y * c2.x)
    + c2.k * c3.k * (c2.x * c3.y + c2.y * c3.x)
    + c3.k * c1.k * (c3.x * c1.y + c3.y * c1.x);
  // Square root of (cross_x + i cross_y).
  let mag = (cross_x * cross_x + cross_y * cross_y).sqrt();
  let half_mag = ((mag + cross_x) * 0.5).max(0.0).sqrt();
  let other_half = (cross_y.abs() / (2.0 * half_mag.max(1e-9))) * cross_y.signum();
  let sx = sign * 2.0 * half_mag;
  let sy = sign * 2.0 * other_half;
  let x4 = (kxsum + sx) / k4;
  let y4 = (kysum + sy) / k4;
  if !x4.is_finite() || !y4.is_finite() || k4.abs() < 1e-6 {
    return None;
  }
  Some(Circle { k: k4, x: x4, y: y4 })
}

/// Generate the Apollonian gasket up to MAX_CIRCLES by repeated Descartes
/// subdivision. Returns the circle list including the outer enclosing
/// circle.
fn build_gasket() -> Vec<Circle> {
  // Outer enclosing circle: curvature -1 (negative because it contains
  // the gasket from outside), centered at origin, radius 1.
  let outer = Circle { k: -1.0, x: 0.0, y: 0.0 };
  // Three inner mutually tangent unit-like circles arranged equilateral.
  // For three equal circles of curvature k tangent to each other inside
  // a unit circle: 1 + 3k = ±2 sqrt(3 k²) => 1 + 3k = 2 sqrt 3 |k|.
  // Solving: k = (2 sqrt(3) + 3) / -1... easier: known answer is k = (2 + sqrt(3)).
  // Use the symmetric solution: each inner circle has curvature 1 + 2/sqrt(3).
  let k_inner = 1.0 + 2.0 / 3.0_f64.sqrt();
  let r_inner = 1.0 / k_inner;
  let dist = 1.0 - r_inner;
  let c1 = Circle { k: k_inner, x: dist * 0.0, y: dist * 1.0 };
  let c2 = Circle {
    k: k_inner,
    x: dist * (TAU / 3.0 + std::f64::consts::FRAC_PI_2).cos(),
    y: dist * (TAU / 3.0 + std::f64::consts::FRAC_PI_2).sin(),
  };
  let c3 = Circle {
    k: k_inner,
    x: dist * (2.0 * TAU / 3.0 + std::f64::consts::FRAC_PI_2).cos(),
    y: dist * (2.0 * TAU / 3.0 + std::f64::consts::FRAC_PI_2).sin(),
  };

  let mut circles = vec![outer, c1, c2, c3];
  // BFS-style subdivision: maintain a queue of triples to expand. Each
  // triple has a "known fourth" curvature to skip (the one we came
  // from), so Descartes gives the other curvature.
  let mut queue: Vec<(usize, usize, usize, usize)> = vec![
    // (i1, i2, i3, known_fourth_idx)
    (0, 1, 2, 3),
    (0, 1, 3, 2),
    (0, 2, 3, 1),
    (1, 2, 3, 0),
  ];

  while let Some((i1, i2, i3, known)) = queue.pop() {
    if circles.len() >= MAX_CIRCLES {
      break;
    }
    let c1 = circles[i1];
    let c2 = circles[i2];
    let c3 = circles[i3];
    let known_c = circles[known];
    // The two Descartes solutions sum to 2(k1+k2+k3). The "other" one is
    // 2(k1+k2+k3) - k_known.
    let target_k = 2.0 * (c1.k + c2.k + c3.k) - known_c.k;
    // Find which sign gives this curvature.
    if let Some(new_c) = fourth_circle(c1, c2, c3, 1.0).filter(|c| (c.k - target_k).abs() < 1e-6) {
      try_add(&mut circles, &mut queue, new_c, i1, i2, i3);
    } else if let Some(new_c) = fourth_circle(c1, c2, c3, -1.0).filter(|c| (c.k - target_k).abs() < 1e-6) {
      try_add(&mut circles, &mut queue, new_c, i1, i2, i3);
    }
  }

  circles
}

fn try_add(
  circles: &mut Vec<Circle>,
  queue: &mut Vec<(usize, usize, usize, usize)>,
  new_c: Circle,
  i1: usize,
  i2: usize,
  i3: usize,
) {
  if new_c.k.abs() > 1.0 / MIN_RADIUS {
    return;
  }
  let new_idx = circles.len();
  circles.push(new_c);
  // The new circle is mutually tangent to c1, c2, c3. Each triple
  // (new, ca, cb) for ca, cb in {c1, c2, c3} has a fourth circle to
  // expand.
  queue.push((new_idx, i1, i2, i3));
  queue.push((new_idx, i1, i3, i2));
  queue.push((new_idx, i2, i3, i1));
}

pub struct Apollonian {
  gasket: Vec<Circle>,
}

impl Default for Apollonian {
  fn default() -> Self {
    Self { gasket: build_gasket() }
  }
}

impl Animation for Apollonian {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let w = ctx.width;
    let h = ctx.height;
    let ax = w as f64 / (h as f64 * 2.0).max(1.0);
    let contrast = ctx.options.contrast;
    let t = ctx.elapsed;
    let dw = (w.saturating_sub(1)).max(1) as f64;
    let dh = (h.saturating_sub(1)).max(1) as f64;

    // Slow drift: rotate the whole gasket about its centroid and breathe
    // a small zoom so the static structure feels alive.
    let theta = t * 0.08;
    let zoom = 1.0 + 0.06 * (t * 0.05).sin();
    let view_scale = 0.42 / zoom;
    let (ct, st) = (theta.cos(), theta.sin());

    let mut grid = vec![0.0_f64; w * h];

    for row in 0..h {
      let v = row as f64 / dh - 0.5;
      let base = row * w;
      for col in 0..w {
        let u = (col as f64 / dw - 0.5) * ax;
        // Inverse rotate the screen point so the gasket appears to rotate.
        let zx = (ct * u + st * v) / view_scale;
        let zy = (-st * u + ct * v) / view_scale;

        // For each circle, contribute a rim brightness based on distance
        // to that circle's boundary. Inner circles (radius small) get a
        // sharper, brighter rim; outer enclosure gets a softer hint.
        let mut value = 0.0_f64;
        for c in &self.gasket {
          let dx = zx - c.x;
          let dy = zy - c.y;
          let dist = (dx * dx + dy * dy).sqrt();
          let r = c.radius();
          let band = (r * 0.10).max(0.003);
          // Gaussian rim centred on |z - c| = r.
          let rim = (-((dist - r).powi(2)) / (band * band)).exp();
          // Smaller circles brighter so the fractal detail reads.
          let weight = (0.30 + 0.55 * r.powf(0.4)).min(0.95);
          value = value.max(rim * weight);
        }
        grid[base + col] = clamp(value * contrast);
      }
    }
    render_field(ctx, &grid, TH, &STYLE, out);
  }
}
