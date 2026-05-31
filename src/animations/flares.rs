use crate::animation::{Animation, FrameContext};
use crate::core::{clamp, render_field, smoothstep, FieldStyle};

const STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "fire" };
const TH: &[(f64, char)] = &[
  (0.13, ' '), (0.22, '.'), (0.31, ':'), (0.42, '-'), (0.54, '='),
  (0.67, '+'), (0.80, '*'), (0.92, '#'), (1.01, '%'),
];

// (offset, speed, reach, width_factor)
const PLUMES: &[(f64, f64, f64, f64)] = &[
  (0.2, 0.47, 1.38, 0.055),
  (1.6, -0.31, 1.22, 0.045),
  (2.8, 0.22, 1.50, 0.060),
  (4.3, -0.18, 1.32, 0.050),
  (5.2, 0.36, 1.18, 0.040),
];
// (offset, speed, arch_height)
const LOOPS: &[(f64, f64, f64)] = &[
  (0.8, 0.20, 0.34), (2.4, -0.16, 0.28), (3.7, 0.13, 0.42),
];

pub struct Flares;

impl Animation for Flares {
  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    let w = ctx.width; let h = ctx.height;
    let mut grid = vec![0.0_f64; w * h];
    let radius = (w.min(h * 2) as f64 * 0.34).max(1.0);
    let t = ctx.elapsed * 0.9;
    let sin_t023 = 0.10 * (t * 0.23).sin();
    let contrast = ctx.options.contrast;
    for row in 0..h {
      let py = ((row as f64 - (h as f64 - 1.0) * 0.5) * 2.0) / radius;
      let base = row * w;
      for col in 0..w {
        let px = (col as f64 - (w as f64 - 1.0) * 0.5) / radius;
        let r = (px * px + py * py).sqrt();
        let theta = py.atan2(px) + sin_t023;
        let limb = smoothstep(1.04, 0.90, r);
        let corona = ((-3.3 * (r - 0.92).max(0.0)).exp() - 0.12).max(0.0);
        let surface = limb * (
          0.20
          + 0.13 * (9.0 * theta + 1.4 * (3.0 * theta - 0.4 * t).sin()).sin().abs()
          + 0.09 * (17.0 * theta + 0.55 * t).sin().abs()
        );
        let radial = (r - 0.72).max(0.0);
        let mut plume = 0.0_f64;
        for &(offset, speed, reach, wf) in PLUMES {
          let center = offset + speed * t + 0.28 * (1.7 * radial + t * 0.33 + offset).sin();
          let ang = (theta - center).sin().atan2((theta - center).cos());
          let strand = (-(ang * ang) / wf).exp();
          let gate = smoothstep(0.76, 1.02, r) * smoothstep(reach, 0.94, r);
          let texture = 0.60 + 0.40 * (18.0 * radial - 2.4 * t + offset).sin().abs();
          plume += strand * gate * texture;
        }
        let mut loops = 0.0_f64;
        for &(offset, speed, arch) in LOOPS {
          let center = offset + speed * t;
          let ang = (theta - center).sin().atan2((theta - center).cos());
          let archv = 1.00 + arch * ((1.0 - ang.abs() / 0.55).max(0.0) * std::f64::consts::PI).sin();
          let line = (-((r - archv).powi(2)) / 0.0025).exp();
          let span = smoothstep(0.62, 0.08, ang.abs());
          let pulse = 0.55 + 0.45 * (13.0 * ang + 1.5 * t + offset).sin().abs();
          loops += line * span * pulse;
        }
        let sparks = 0.10 * (37.0 * r + 11.0 * theta - 2.2 * t).sin().max(0.0);
        let exterior = smoothstep(0.82, 1.02, r);
        let texture = surface + exterior * (0.22 * corona + 0.56 * plume + 0.52 * loops + sparks);
        let vignette = (1.0 - 0.07 * (px.abs() + py.abs())).max(0.0);
        grid[base + col] = clamp(texture * vignette * contrast);
      }
    }
    render_field(ctx, &grid, TH, &STYLE, out);
  }
}
