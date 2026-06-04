//! DNA double helix as a 3D parametric curve that rotates around camera-Y
//! and tilts on camera-X, so the viewer sees the helix from continuously
//! changing angles instead of the previous fixed-axis vertical view.
//!
//! Two strands, opposite phases on the same helix; rungs connect the strands
//! every quarter-turn. Brightness is depth-shaded -- points farther from the
//! camera are dimmer, points closer brighter -- which gives the 3D vividness
//! the previous "twist only" version was missing.

use std::f64::consts::TAU;

use crate::animation::{Animation, FrameContext};
use crate::core::{render_glyph_field, FieldStyle};

const STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "ocean" };

const N_TURNS: f64 = 5.0;
const PTS_PER_TURN: usize = 80;

pub struct Dna;

impl Animation for Dna {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let w = ctx.width;
    let h = ctx.height;
    let ax_correction = w as f64 / (h as f64 * 2.0).max(1.0);
    let cx = w as f64 * 0.5;
    let cy = h as f64 * 0.5;
    let radius = (w as f64 * 0.18).max(3.0);
    let length = h as f64 * 0.85;
    let t = ctx.elapsed;
    // Helix spins around its own long axis (the "twist").
    let twist_phase = t * 1.2;
    // Camera rotation. Continuous Y rotation tumbles the helix; a slower X
    // tilt nods it forward / backward so you see it from oblique angles too.
    let camera_y_rot = t * 0.32;
    let camera_x_rot = 0.38 * (t * 0.18).sin();
    let (cy_rot_c, cy_rot_s) = (camera_y_rot.cos(), camera_y_rot.sin());
    let (cx_rot_c, cx_rot_s) = (camera_x_rot.cos(), camera_x_rot.sin());

    let n_pts = (N_TURNS * PTS_PER_TURN as f64) as usize;

    let mut grid = vec![0.0_f64; w * h];
    let mut glyphs = vec![' '; w * h];
    let put = |grid: &mut Vec<f64>, glyphs: &mut Vec<char>, col: f64, row: f64, value: f64, ch: char| {
      let c = col.round() as i64;
      let r = row.round() as i64;
      if c >= 0 && (c as usize) < w && r >= 0 && (r as usize) < h {
        let idx = r as usize * w + c as usize;
        if grid[idx] < value {
          grid[idx] = value;
          glyphs[idx] = ch;
        }
      }
    };

    // Project a 3D point to screen with the current camera, returning
    // (screen_col, screen_row, depth) where depth ∈ ~[-1, 1].
    let project = |p: (f64, f64, f64)| -> (f64, f64, f64) {
      // Rotate around Y first (tumble), then X (nod).
      let q1 = (p.0 * cy_rot_c + p.2 * cy_rot_s, p.1, -p.0 * cy_rot_s + p.2 * cy_rot_c);
      let q2 = (q1.0, q1.1 * cx_rot_c - q1.2 * cx_rot_s, q1.1 * cx_rot_s + q1.2 * cx_rot_c);
      // Orthographic project. Apply terminal cell-aspect correction so the
      // helix does not look horizontally stretched.
      let depth_norm = q2.2 / radius.max(1.0);
      (cx + q2.0 / ax_correction, cy + q2.1, depth_norm)
    };

    for k in 0..n_pts {
      let u = k as f64 / n_pts as f64;
      let helix_angle = TAU * N_TURNS * u + twist_phase;
      let local_y = (u - 0.5) * length;
      let strand_a_local = (radius * helix_angle.cos(), local_y, radius * helix_angle.sin());
      let strand_b_local = (-strand_a_local.0, local_y, -strand_a_local.2);

      let sa = project(strand_a_local);
      let sb = project(strand_b_local);

      // Depth shading: brighter when closer to the viewer (larger z after
      // rotation = "in front of" the centre).
      let bright_a = (0.40 + 0.55 * (sa.2 * 0.5 + 0.5)).clamp(0.18, 1.0);
      let bright_b = (0.40 + 0.55 * (sb.2 * 0.5 + 0.5)).clamp(0.18, 1.0);

      // Glyph encodes which side of the centreline (front / back of helix).
      let glyph_a = if sa.2 > 0.0 { '#' } else { '.' };
      let glyph_b = if sb.2 > 0.0 { '#' } else { '.' };
      put(&mut grid, &mut glyphs, sa.0, sa.1, bright_a, glyph_a);
      put(&mut grid, &mut glyphs, sb.0, sb.1, bright_b, glyph_b);

      // Rung every quarter turn: draw a line connecting strand_a -> strand_b
      // in screen space, depth-shaded along its length.
      if k % (PTS_PER_TURN / 4) == 0 {
        let dx = sb.0 - sa.0;
        let dy = sb.1 - sa.1;
        let dz = sb.2 - sa.2;
        let steps = (dx.abs() + dy.abs()).ceil() as i64;
        if steps >= 2 {
          for i in 1..steps {
            let s = i as f64 / steps as f64;
            let rx = sa.0 + dx * s;
            let ry = sa.1 + dy * s;
            let rz = sa.2 + dz * s;
            let rb = (0.28 + 0.32 * (rz * 0.5 + 0.5)).clamp(0.16, 0.85);
            let rg = if i % 2 == 0 { '=' } else { '-' };
            put(&mut grid, &mut glyphs, rx, ry, rb, rg);
          }
        }
      }
    }
    render_glyph_field(ctx, &grid, &glyphs, &STYLE, out);
  }
}
