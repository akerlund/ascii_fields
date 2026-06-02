use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, render_glyph_field, FieldStyle};

const STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "ice" };
const DIAGRAM_SECONDS: f64 = 4.5;
const TRAVEL: f64 = 2.0;

#[derive(Clone, Copy)]
enum Kind {
  Fermion,
  Photon,
  Gluon,
}
enum Element {
  Vertex(f64, f64),
  Label(f64, f64, &'static str),
  Segment(Kind, f64, f64, f64, f64),
}

fn diagrams() -> Vec<Vec<Element>> {
  use Element::*;
  use Kind::*;
  vec![
    vec![
      Label(0.02, 0.18, "e-"),
      Label(0.02, 0.82, "e+"),
      Segment(Fermion, 0.06, 0.20, 0.40, 0.50),
      Segment(Fermion, 0.06, 0.80, 0.40, 0.50),
      Vertex(0.40, 0.50),
      Segment(Photon, 0.40, 0.50, 0.60, 0.50),
      Vertex(0.60, 0.50),
      Segment(Fermion, 0.60, 0.50, 0.94, 0.20),
      Segment(Fermion, 0.60, 0.50, 0.94, 0.80),
      Label(0.95, 0.18, "u-"),
      Label(0.95, 0.82, "u+"),
    ],
    vec![
      Label(0.02, 0.30, "y"),
      Label(0.02, 0.86, "e-"),
      Segment(Photon, 0.06, 0.28, 0.34, 0.62),
      Segment(Fermion, 0.06, 0.86, 0.34, 0.62),
      Vertex(0.34, 0.62),
      Segment(Fermion, 0.34, 0.62, 0.66, 0.62),
      Vertex(0.66, 0.62),
      Segment(Photon, 0.66, 0.62, 0.94, 0.28),
      Segment(Fermion, 0.66, 0.62, 0.94, 0.86),
      Label(0.95, 0.30, "y"),
      Label(0.95, 0.86, "e-"),
    ],
    vec![
      Label(0.02, 0.18, "q"),
      Label(0.02, 0.82, "q"),
      Segment(Fermion, 0.06, 0.20, 0.42, 0.32),
      Segment(Fermion, 0.06, 0.80, 0.42, 0.68),
      Vertex(0.42, 0.32),
      Vertex(0.42, 0.68),
      Segment(Gluon, 0.42, 0.32, 0.42, 0.68),
      Segment(Fermion, 0.42, 0.32, 0.94, 0.20),
      Segment(Fermion, 0.42, 0.68, 0.94, 0.80),
      Label(0.95, 0.18, "q"),
      Label(0.95, 0.82, "q"),
    ],
  ]
}

// The `< low || >= high` shape mirrors the geometric reasoning of these angle
// buckets; converting to `!Range::contains` reads worse, not better, here.
#[allow(clippy::manual_range_contains)]
fn line_char(angle: f64) -> char {
  let a = (angle.to_degrees() + 360.0) % 180.0;
  if a < 22.5 || a >= 157.5 {
    '-'
  } else if a < 67.5 {
    '\\'
  } else if a < 112.5 {
    '|'
  } else {
    '/'
  }
}
#[allow(clippy::manual_range_contains)]
fn arrow_char(angle: f64) -> char {
  let d = (angle.to_degrees() + 360.0) % 360.0;
  if d < 45.0 || d >= 315.0 {
    '>'
  } else if d < 135.0 {
    'v'
  } else if d < 225.0 {
    '<'
  } else {
    '^'
  }
}

pub struct Feynman {
  diagrams: Vec<Vec<Element>>,
}
impl Default for Feynman {
  fn default() -> Self {
    Self { diagrams: diagrams() }
  }
}

impl Animation for Feynman {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let w = ctx.width;
    let h = ctx.height;
    let mut grid = vec![0.0_f64; w * h];
    let mut glyphs = vec![' '; w * h];
    let didx = ((ctx.elapsed / DIAGRAM_SECONDS) as usize) % self.diagrams.len();
    let tpos = (ctx.elapsed % TRAVEL) / TRAVEL;
    let put = |grid: &mut Vec<f64>, glyphs: &mut Vec<char>, x: f64, y: f64, value: f64, ch: char| {
      let c = x.round() as i64;
      let r = y.round() as i64;
      if c >= 0 && (c as usize) < w && r >= 0 && (r as usize) < h {
        let idx = (r as usize) * w + (c as usize);
        if value >= grid[idx] {
          grid[idx] = value;
          glyphs[idx] = ch;
        }
      }
    };
    let dw = (w.saturating_sub(1)).max(1) as f64;
    let dh = (h.saturating_sub(1)).max(1) as f64;
    for el in &self.diagrams[didx] {
      match *el {
        Element::Vertex(x, y) => {
          put(&mut grid, &mut glyphs, x * dw, y * dh, 1.0, '@');
        }
        Element::Label(x, y, text) => {
          let col0 = (x * dw) as i64;
          for (k, ch) in text.chars().enumerate() {
            put(&mut grid, &mut glyphs, (col0 + k as i64) as f64, y * dh, 0.8, ch);
          }
        }
        Element::Segment(kind, x0, y0, x1, y1) => {
          let cx0 = x0 * dw;
          let cy0 = y0 * dh;
          let cx1 = x1 * dw;
          let cy1 = y1 * dh;
          let dx = cx1 - cx0;
          let dy = cy1 - cy0;
          let length = (dx * dx + dy * dy).sqrt().max(1.0);
          let steps = (length as i64).max(2);
          let perp_x = -dy / length;
          let perp_y = dx / length;
          let angle = dy.atan2(dx);
          let base_char = line_char(angle);
          for s in 0..=steps {
            let f = s as f64 / steps as f64;
            let mut x = cx0 + dx * f;
            let mut y = cy0 + dy * f;
            let ch = match kind {
              Kind::Photon => {
                let wig = 1.3 * (s as f64 * 0.7).sin();
                x += perp_x * wig;
                y += perp_y * wig;
                '~'
              }
              Kind::Gluon => {
                let wig = 1.6 * (s as f64 * 1.0).sin();
                x += perp_x * wig;
                y += perp_y * wig;
                'o'
              }
              Kind::Fermion => base_char,
            };
            let pulse = (-((f - tpos).powi(2)) / 0.01).exp();
            put(&mut grid, &mut glyphs, x, y, clamp(0.32 + 0.75 * pulse), ch);
          }
          if matches!(kind, Kind::Fermion) {
            put(&mut grid, &mut glyphs, cx0 + dx * 0.55, cy0 + dy * 0.55, 1.0, arrow_char(angle));
          }
        }
      }
    }
    render_glyph_field(ctx, &grid, &glyphs, &STYLE, out);
  }
}
