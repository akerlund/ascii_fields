//! Continuous Feynman-diagram-style particle cascade.
//!
//! Old: three static, hand-authored diagrams cycled every 4.5 seconds with a
//! pulse travelling along each edge. User wanted a "continuous chain reaction".
//!
//! New: live particle simulation. Incoming particles enter from the screen
//! edges at jittered intervals. Each particle has a fixed lifetime; on decay
//! it becomes a flashing vertex and emits 2-3 secondary particles at random
//! angles. Secondaries decay further, producing an open-ended cascade. Each
//! particle paints a short straight-line worldline as a trail of type-specific
//! glyphs (photons ~, gluons o, fermions line chars + arrowhead), with the
//! head brightest and the tail fading.

use std::f64::consts::TAU;

use rand::{Rng, SeedableRng};
use rand_pcg::Pcg32;

use crate::animation::{Animation, FrameContext};
use crate::core::{render_glyph_field, FieldStyle};

// Spectrum palette + per-particle brightness offsets give each particle
// type its own colour band: fermions sit in the mid range (greens/yellows),
// photons up in the warm end (orange/red), gluons down in the cool end
// (violet/blue). Brightness gradient inside each particle (head -> tail)
// gives a small shimmer within the colour band so the trail still reads
// as motion.
const STYLE: FieldStyle = FieldStyle { gray_lo: 234, gray_hi: 255, default_theme: "spectrum" };

/// Per-type brightness centre: the head of a particle of this kind hits
/// here, the trail tail fades to roughly this minus 0.20.
fn type_brightness(kind: Kind) -> f64 {
  match kind {
    Kind::Gluon => 0.22,
    Kind::Fermion => 0.55,
    Kind::Photon => 0.90,
  }
}

const TRAIL_SECONDS: f64 = 0.6;
const TRAIL_STEPS: usize = 12;
const VERTEX_FLASH_SECONDS: f64 = 0.55;
const MAX_PARTICLES: usize = 80;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
  Fermion,
  Photon,
  Gluon,
}

fn pick_kind(rng: &mut Pcg32) -> Kind {
  match rng.gen_range(0..6) {
    0..=2 => Kind::Fermion, // half the time -- gives the line/arrow look
    3..=4 => Kind::Photon,
    _ => Kind::Gluon,
  }
}

#[derive(Clone, Copy)]
struct Particle {
  kind: Kind,
  x: f64,
  y: f64,
  vx: f64,
  vy: f64,
  t0: f64,
  lifetime: f64,
}

#[derive(Clone, Copy)]
struct VertexFlash {
  x: f64,
  y: f64,
  t0: f64,
}

pub struct Feynman {
  particles: Vec<Particle>,
  flashes: Vec<VertexFlash>,
  next_spawn: f64,
  last: f64,
  rng: Pcg32,
}

impl Default for Feynman {
  fn default() -> Self {
    Self {
      particles: Vec::new(),
      flashes: Vec::new(),
      next_spawn: 0.0,
      last: 0.0,
      rng: Pcg32::seed_from_u64(0xFE17_BABE_F00D_DEAD),
    }
  }
}

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

impl Feynman {
  fn spawn_incoming(&mut self, t0: f64) {
    // Pick a screen edge and a target point on the opposite side.
    let from_edge = self.rng.gen_range(0..4);
    let (x, y) = match from_edge {
      0 => (0.02, self.rng.gen_range(0.15..0.85)), // left
      1 => (0.98, self.rng.gen_range(0.15..0.85)), // right
      2 => (self.rng.gen_range(0.15..0.85), 0.02), // top
      _ => (self.rng.gen_range(0.15..0.85), 0.98), // bottom
    };
    let toward_x: f64 = self.rng.gen_range(0.30..0.70);
    let toward_y: f64 = self.rng.gen_range(0.30..0.70);
    let dx = toward_x - x;
    let dy = toward_y - y;
    let len = (dx * dx + dy * dy).sqrt().max(1e-6);
    let speed = self.rng.gen_range(0.10..0.20);
    self.particles.push(Particle {
      kind: pick_kind(&mut self.rng),
      x,
      y,
      vx: dx / len * speed,
      vy: dy / len * speed,
      t0,
      lifetime: self.rng.gen_range(2.0..4.5),
    });
  }
}

impl Animation for Feynman {
  fn status(&self) -> Option<String> {
    let (mut f, mut p, mut g) = (0usize, 0usize, 0usize);
    for particle in &self.particles {
      match particle.kind {
        Kind::Fermion => f += 1,
        Kind::Photon => p += 1,
        Kind::Gluon => g += 1,
      }
    }
    Some(format!("{} particles ({}f / {}γ / {}g)", self.particles.len(), f, p, g))
  }

  fn render(&mut self, ctx: &FrameContext, out: &mut String) {
    // Reset on time rewind (export, HUD scrub).
    if ctx.elapsed < self.last {
      self.particles.clear();
      self.flashes.clear();
      self.next_spawn = 0.0;
    }
    let dt = (ctx.elapsed - self.last).max(0.0);
    self.last = ctx.elapsed;

    // Spawn incoming particles at jittered intervals so the cascade keeps
    // being fed (otherwise everything decays away in ~10 seconds).
    while ctx.elapsed >= self.next_spawn {
      self.spawn_incoming(self.next_spawn);
      // Avoid the borrow conflict of rng + self by sampling the next
      // interval right after.
      let next: f64 = self.rng.gen_range(0.8..2.4);
      self.next_spawn += next;
    }

    // Move particles forward in their straight worldlines.
    for p in &mut self.particles {
      p.x += p.vx * dt;
      p.y += p.vy * dt;
    }

    // Decay: particles whose age exceeds their lifetime become vertex
    // flashes and emit 2-3 secondaries at random angles + speeds.
    let mut idx = 0;
    while idx < self.particles.len() {
      let age = ctx.elapsed - self.particles[idx].t0;
      if age >= self.particles[idx].lifetime {
        let p = self.particles.swap_remove(idx);
        // Skip the cascade if the decay point is off-screen -- no point
        // emitting particles the viewer can't see.
        if (0.0..1.0).contains(&p.x) && (0.0..1.0).contains(&p.y) {
          self.flashes.push(VertexFlash { x: p.x, y: p.y, t0: ctx.elapsed });
          let n_secondaries = self.rng.gen_range(2..=3);
          for _ in 0..n_secondaries {
            let angle = self.rng.gen_range(0.0..TAU);
            let speed: f64 = self.rng.gen_range(0.08..0.18);
            self.particles.push(Particle {
              kind: pick_kind(&mut self.rng),
              x: p.x,
              y: p.y,
              vx: angle.cos() * speed,
              vy: angle.sin() * speed,
              t0: ctx.elapsed,
              // Secondaries live shorter than primaries so the cascade
              // remains finite per spawn -- otherwise the screen would
              // fill up and stay full forever.
              lifetime: self.rng.gen_range(0.9..2.4),
            });
          }
        }
      } else {
        idx += 1;
      }
    }

    // Cull off-screen + cap counts.
    self.particles.retain(|p| (-0.15..1.15).contains(&p.x) && (-0.15..1.15).contains(&p.y));
    self.flashes.retain(|f| ctx.elapsed - f.t0 < VERTEX_FLASH_SECONDS);
    while self.particles.len() > MAX_PARTICLES {
      self.particles.remove(0);
    }

    let w = ctx.width;
    let h = ctx.height;
    let dw = (w.saturating_sub(1)).max(1) as f64;
    let dh = (h.saturating_sub(1)).max(1) as f64;
    let mut grid = vec![0.0_f64; w * h];
    let mut glyphs = vec![' '; w * h];
    let put = |grid: &mut Vec<f64>, glyphs: &mut Vec<char>, col: f64, row: f64, value: f64, ch: char| {
      let c = col.round() as i64;
      let r = row.round() as i64;
      if c >= 0 && (c as usize) < w && r >= 0 && (r as usize) < h {
        let i = r as usize * w + c as usize;
        if value > grid[i] {
          grid[i] = value;
          glyphs[i] = ch;
        }
      }
    };

    // Draw each particle's recent worldline as a trail of type-specific
    // glyphs. Head bright, tail fading.
    for p in &self.particles {
      let age = ctx.elapsed - p.t0;
      let trail_age = age.min(TRAIL_SECONDS);
      let angle = p.vy.atan2(p.vx);
      let base_char = match p.kind {
        Kind::Photon => '~',
        Kind::Gluon => 'o',
        Kind::Fermion => line_char(angle),
      };
      // Photons / gluons get a sinusoidal wiggle along their length so the
      // visible difference between types stays obvious.
      let dx = -p.vx;
      let dy = -p.vy;
      let perp_x = -p.vy;
      let perp_y = p.vx;
      let pspeed = (perp_x * perp_x + perp_y * perp_y).sqrt().max(1e-6);
      let perp_nx = perp_x / pspeed;
      let perp_ny = perp_y / pspeed;
      let kind_brightness = type_brightness(p.kind);
      // Trail brightness shimmers in a +/-0.06 band around kind_brightness
      // so each particle is recognisable as its colour without losing
      // motion cues.
      for s in 0..TRAIL_STEPS {
        let frac = s as f64 / TRAIL_STEPS as f64;
        let dt_back = frac * trail_age;
        let mut tx = p.x + dx * dt_back;
        let mut ty = p.y + dy * dt_back;
        let wiggle = match p.kind {
          Kind::Photon => 0.012 * (s as f64 * 0.9).sin(),
          Kind::Gluon => 0.018 * (s as f64 * 1.2).sin(),
          Kind::Fermion => 0.0,
        };
        tx += perp_nx * wiggle;
        ty += perp_ny * wiggle;
        // Tail dims slightly within the colour band so the trail still
        // has a brightness gradient inside its own hue.
        let brightness = (kind_brightness - 0.10 * frac).clamp(0.05, 0.99);
        put(&mut grid, &mut glyphs, tx * dw, ty * dh, brightness, base_char);
      }
      // Head: a bit brighter than the trail so the leading edge pops,
      // still within the kind's colour band.
      let head_ch = match p.kind {
        Kind::Fermion => arrow_char(angle),
        Kind::Photon => '*',
        Kind::Gluon => '@',
      };
      let head_brightness = (kind_brightness + 0.06).clamp(0.05, 0.99);
      put(&mut grid, &mut glyphs, p.x * dw, p.y * dh, head_brightness, head_ch);
    }

    // Vertex flashes: bright @ at the decay point that fades within
    // VERTEX_FLASH_SECONDS.
    for f in &self.flashes {
      let age = ctx.elapsed - f.t0;
      let brightness = (1.0 - age / VERTEX_FLASH_SECONDS).clamp(0.0, 1.0);
      put(&mut grid, &mut glyphs, f.x * dw, f.y * dh, brightness, '@');
    }

    render_glyph_field(ctx, &grid, &glyphs, &STYLE, out);
  }
}
