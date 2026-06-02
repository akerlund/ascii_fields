use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, render_field, FieldStyle};
use crate::noise::{fbm, star_noise};

const STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "ice" };
const TH: &[(f64, char)] = &[
  (0.06, ' '),
  (0.16, '.'),
  (0.30, ':'),
  (0.45, '-'),
  (0.60, '+'),
  (0.74, '*'),
  (0.86, 'o'),
  (0.94, '#'),
  (1.01, '@'),
];

pub struct NightSky;

impl Animation for NightSky {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let w = ctx.width;
    let h = ctx.height;
    let density = ctx.options.scale.max(0.4);
    let mut grid = vec![0.0_f64; w * h];
    let band_angle = 0.5_f64;
    let ca = band_angle.cos();
    let sa = band_angle.sin();
    let meteor = meteor_state(ctx.elapsed, w, h);
    let dw = (w.saturating_sub(1)).max(1) as f64;
    let dh = (h.saturating_sub(1)).max(1) as f64;
    for row in 0..h {
      let ny = (row as f64 / dh - 0.5) * 2.0;
      let base = row * w;
      for col in 0..w {
        let nx = (col as f64 / dw - 0.5) * 2.0;
        let across = nx * sa - ny * ca;
        let haze = (-(across * across) / 0.12).exp();
        let milky = haze * (0.10 + 0.26 * fbm(nx * 3.0 + 4.0, ny * 3.0, 4));
        let mut level = 0.04 + milky;
        let n = star_noise(col as i64, row as i64);
        if n > 0.991 {
          let mag = 0.55 + 0.45 * star_noise(col as i64 + 3, row as i64 + 11);
          let twinkle = 0.6 + 0.4 * (ctx.elapsed * (1.5 + 4.0 * n) + col as f64 * 0.7 + row as f64).sin();
          level = level.max(mag * twinkle * (0.7 + 0.3 * density));
        } else if n > 0.975 {
          level = level.max(0.22 + 0.10 * (ctx.elapsed * 2.0 + col as f64).sin());
        }
        if let Some(m) = &meteor {
          let mb = meteor_brightness(m, col as f64, row as f64);
          if mb > level {
            level = mb;
          }
        }
        grid[base + col] = clamp(level);
      }
    }
    render_field(ctx, &grid, TH, &STYLE, out);
  }
}

struct Meteor {
  head_x: f64,
  head_y: f64,
  dx: f64,
  dy: f64,
  fade: f64,
}

fn meteor_state(elapsed: f64, w: usize, h: usize) -> Option<Meteor> {
  let period = 6.5;
  let duration = 1.1;
  let idx = (elapsed / period) as i64;
  let local = elapsed - idx as f64 * period;
  if local > duration {
    return None;
  }
  let r = star_noise(idx * 17 + 3, idx * 5 + 1);
  let r2 = star_noise(idx * 9 + 7, idx * 13 + 2);
  let start_x = r * w as f64;
  let start_y = r2 * h as f64 * 0.5;
  let dx = (0.7 + 0.5 * r2) * w as f64;
  let dy = (0.4 + 0.4 * r) * h as f64;
  let p = local / duration;
  Some(Meteor { head_x: start_x + dx * p, head_y: start_y + dy * p, dx, dy, fade: 1.0 - p })
}

// `along > 0.6 || along < -10.0` mirrors "outside the meteor's body along
// the trail direction"; converting to !RangeInclusive::contains hides intent.
#[allow(clippy::manual_range_contains)]
fn meteor_brightness(m: &Meteor, col: f64, row: f64) -> f64 {
  let length = (m.dx * m.dx + m.dy * m.dy).sqrt();
  let ux = m.dx / length;
  let uy = m.dy / length;
  let rx = col - m.head_x;
  let ry = row - m.head_y;
  let along = rx * ux + ry * uy;
  let perp = (rx * uy - ry * ux).abs();
  if along > 0.6 || along < -10.0 || perp > 1.4 {
    return 0.0;
  }
  let tail = if along < 0.0 { (along * 0.45).exp() } else { 1.0 };
  let core = (-(perp * perp) / 0.5).exp();
  (tail * core * m.fade * 1.2).clamp(0.0, 1.0)
}
