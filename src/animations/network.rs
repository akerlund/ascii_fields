use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, render_glyph_field, FieldStyle};

const STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "copper" };

// (x, y, kind): kind in {R router, S switch, E endpoint, o ordinary}
const NODES: &[(f64, f64, char)] = &[
  (0.04, 0.10, 'E'),
  (0.18, 0.06, 'o'),
  (0.38, 0.08, 'S'),
  (0.62, 0.06, 'o'),
  (0.82, 0.10, 'S'),
  (0.96, 0.18, 'E'),
  (0.08, 0.36, 'o'),
  (0.24, 0.30, 'R'),
  (0.46, 0.33, 'o'),
  (0.66, 0.28, 'R'),
  (0.88, 0.38, 'o'),
  (0.98, 0.52, 'E'),
  (0.03, 0.66, 'E'),
  (0.20, 0.70, 'o'),
  (0.42, 0.63, 'R'),
  (0.60, 0.70, 'o'),
  (0.78, 0.64, 'R'),
  (0.94, 0.78, 'o'),
  (0.14, 0.92, 'E'),
  (0.34, 0.86, 'o'),
  (0.55, 0.92, 'S'),
  (0.76, 0.88, 'o'),
  (0.96, 0.94, 'E'),
];
const EDGES: &[(usize, usize)] = &[
  (0, 1),
  (1, 2),
  (2, 3),
  (3, 4),
  (4, 5),
  (0, 6),
  (1, 7),
  (2, 7),
  (2, 8),
  (3, 8),
  (4, 9),
  (5, 10),
  (5, 11),
  (6, 7),
  (7, 8),
  (8, 9),
  (9, 10),
  (10, 11),
  (6, 12),
  (7, 13),
  (8, 14),
  (9, 15),
  (10, 16),
  (11, 17),
  (12, 13),
  (13, 14),
  (14, 15),
  (15, 16),
  (16, 17),
  (12, 18),
  (13, 19),
  (14, 19),
  (14, 20),
  (15, 20),
  (16, 21),
  (17, 22),
  (18, 19),
  (19, 20),
  (20, 21),
  (21, 22),
  (7, 14),
  (9, 14),
  (9, 16),
  (14, 20),
  (4, 16),
];
const PACKETS: &[(char, f64, f64)] = &[('@', 1.00, 0.085), ('*', 0.86, 0.115), ('+', 0.72, 0.145)];

fn line_char(dx: f64, dy: f64) -> char {
  if dx.abs() > dy.abs() * 1.8 {
    '-'
  } else if dy.abs() > dx.abs() * 1.8 {
    '|'
  } else if dx * dy > 0.0 {
    '\\'
  } else {
    '/'
  }
}

pub struct Network;

impl Animation for Network {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let w = ctx.width;
    let h = ctx.height;
    let mut grid = vec![0.0_f64; w * h];
    let mut glyphs = vec![' '; w * h];
    if w == 0 || h == 0 {
      return;
    }
    let dw = (w - 1) as f64;
    let dh = (h - 1) as f64;
    let put = |grid: &mut Vec<f64>, glyphs: &mut Vec<char>, col: i64, row: i64, value: f64, ch: char| {
      if col >= 0 && (col as usize) < w && row >= 0 && (row as usize) < h {
        let idx = (row as usize) * w + (col as usize);
        if value >= grid[idx] {
          grid[idx] = value;
          glyphs[idx] = ch;
        }
      }
    };
    let point = |i: usize| -> (i64, i64) {
      let n = NODES[i];
      ((n.0 * dw).round() as i64, (n.1 * dh).round() as i64)
    };
    for (edge_idx, &(a, b)) in EDGES.iter().enumerate() {
      let (c0, r0) = point(a);
      let (c1, r1) = point(b);
      let dx = (c1 - c0) as f64;
      let dy = (r1 - r0) as f64;
      let length = (dx * dx + dy * dy).sqrt().max(1.0);
      let steps = length as i64;
      let ch = line_char(dx, dy);
      let load = 0.17 + 0.06 * (ctx.elapsed * 0.8 + edge_idx as f64).sin();
      for step in 0..=steps {
        let f = step as f64 / steps as f64;
        put(
          &mut grid,
          &mut glyphs,
          (c0 as f64 + dx * f).round() as i64,
          (r0 as f64 + dy * f).round() as i64,
          load,
          ch,
        );
      }
      let perp_x = -dy / length;
      let perp_y = dx / length;
      for (pi, &(glyph, brightness, speed)) in PACKETS.iter().enumerate() {
        if (edge_idx + pi) % 3 == 2 {
          continue;
        }
        let mut pulse = ((ctx.elapsed * (speed + 0.006 * (edge_idx % 7) as f64)
          + edge_idx as f64 * 0.091
          + pi as f64 * 0.27)
          % 1.0
          + 1.0)
          % 1.0;
        if (edge_idx + pi) % 4 == 0 {
          pulse = 1.0 - pulse;
        }
        let lane = (pi as f64 - 1.0) * 0.6;
        let ox = perp_x * lane;
        let oy = perp_y * lane;
        for tail in 0..4 {
          let f = pulse - tail as f64 * 0.030;
          if !(0.0..=1.0).contains(&f) {
            continue;
          }
          let value = brightness - tail as f64 * 0.13;
          put(
            &mut grid,
            &mut glyphs,
            (c0 as f64 + dx * f + ox).round() as i64,
            (r0 as f64 + dy * f + oy).round() as i64,
            value,
            if tail == 0 { glyph } else { '.' },
          );
        }
      }
    }
    for (idx, &(_, _, kind)) in NODES.iter().enumerate() {
      let (col, row) = point(idx);
      let beat = 0.70 + 0.30 * (ctx.elapsed * 1.5 + idx as f64 * 0.9).sin();
      match kind {
        'R' => {
          put(&mut grid, &mut glyphs, col, row, 1.0, '#');
          for &(dc, dr) in &[(1, 0), (-1, 0), (0, 1), (0, -1)] {
            put(&mut grid, &mut glyphs, col + dc, row + dr, 0.45, '+');
          }
        }
        'S' => {
          put(&mut grid, &mut glyphs, col, row, 0.90 + 0.10 * beat, 'X');
          put(&mut grid, &mut glyphs, col - 1, row, 0.40, '=');
          put(&mut grid, &mut glyphs, col + 1, row, 0.40, '=');
        }
        'E' => {
          put(&mut grid, &mut glyphs, col, row, 0.78 + 0.20 * beat, 'O');
        }
        _ => {
          put(&mut grid, &mut glyphs, col, row, 0.66 + 0.25 * beat, 'o');
          put(&mut grid, &mut glyphs, col, row - 1, 0.24, '.');
        }
      }
    }
    let contrast = ctx.options.contrast;
    for v in grid.iter_mut() {
      *v = clamp(*v * contrast);
    }
    render_glyph_field(ctx, &grid, &glyphs, &STYLE, out);
  }
}
