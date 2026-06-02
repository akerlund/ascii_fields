//! Criterion bench for the rendering hot path. Measures the wall time of one
//! `Animation::render` call for a representative spread of modes, at a fixed
//! frame size and theme. Run with `cargo bench` for a clean baseline, or
//! `cargo bench -- --save-baseline NAME` then `cargo bench -- --baseline NAME`
//! to compare two revisions.

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};

use ascii_fields::animation::{FrameContext, THEME_COLOR_STEPS};
use ascii_fields::options::RenderOptions;
use ascii_fields::registry::{self, ModeInfo};

const BENCH_MODES: &[&str] = &[
  // Parallelized hot density fields (from the rayon perf pass).
  "lava",
  "storm",
  "aurora",
  "clouds",
  "tunnel",
  "whirlpool",
  // Stateful / non-trivial baselines for contrast.
  "plasma",
  "galaxy",
  "rd",
  // Glyph-locked mode (charset cycle no-op, exercises render_glyph_field).
  "cpu",
];

const BENCH_WIDTH: usize = 160;
const BENCH_HEIGHT: usize = 50;

fn mode_info(name: &str) -> &'static ModeInfo {
  registry::info(name).unwrap_or_else(|| panic!("bench mode '{name}' not in registry"))
}

fn options_for(_name: &str) -> RenderOptions {
  let mut opt = RenderOptions::default();
  opt.theme = "scene".to_string();
  opt
}

fn bench_render(c: &mut Criterion) {
  let mut group = c.benchmark_group("render");
  group.sample_size(40);

  for &name in BENCH_MODES {
    let info = mode_info(name);
    let opt = options_for(name);
    let mut anim = (info.factory)();
    let mut out = String::with_capacity(BENCH_WIDTH * BENCH_HEIGHT * 24);

    group.bench_function(name, |b| {
      // Use a moving timestamp so stateful modes (rd, life, dla, ...) advance.
      let mut t = 0.0_f64;
      b.iter(|| {
        let ctx = FrameContext {
          width: BENCH_WIDTH,
          height: BENCH_HEIGHT,
          elapsed: t,
          phase: t,
          color_steps: THEME_COLOR_STEPS,
          options: &opt,
        };
        out.clear();
        anim.render(black_box(&ctx), black_box(&mut out));
        t += 1.0 / 24.0;
      });
    });
  }

  group.finish();
}

criterion_group!(benches, bench_render);
criterion_main!(benches);
