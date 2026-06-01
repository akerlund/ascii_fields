//! Truecolor gradient palettes.
//!
//! `palette_table(name)` returns a `&'static [(u8, u8, u8)]` of 256 entries.
//! Each table is built lazily into its own `OnceLock`, so the hot path is a
//! simple array index -- no mutex, no hash lookup, no `String` allocation.

use std::sync::OnceLock;

type Stops = &'static [(f64, (u8, u8, u8))];

const MONO: Stops = &[(0.0, (24, 24, 24)), (1.0, (255, 255, 255))];
const FIRE: Stops = &[
  (0.0, (0, 0, 0)),
  (0.30, (90, 12, 4)),
  (0.55, (200, 55, 10)),
  (0.78, (255, 150, 25)),
  (0.92, (255, 225, 110)),
  (1.0, (255, 255, 230)),
];
const LAVA: Stops = &[
  (0.0, (5, 0, 0)),
  (0.25, (60, 6, 0)),
  (0.50, (170, 30, 0)),
  (0.72, (240, 95, 10)),
  (0.88, (255, 175, 40)),
  (1.0, (255, 240, 180)),
];
const ICE: Stops = &[
  (0.0, (0, 0, 0)),
  (0.30, (8, 22, 60)),
  (0.55, (20, 80, 150)),
  (0.78, (70, 170, 220)),
  (1.0, (225, 250, 255)),
];
const NEBULA: Stops = &[
  (0.0, (0, 0, 0)),
  (0.28, (40, 8, 70)),
  (0.52, (120, 25, 140)),
  (0.74, (210, 60, 160)),
  (0.90, (250, 150, 200)),
  (1.0, (255, 240, 250)),
];
const AURORA: Stops = &[
  (0.0, (0, 0, 0)),
  (0.30, (4, 40, 35)),
  (0.52, (20, 140, 90)),
  (0.72, (70, 220, 150)),
  (0.88, (160, 250, 210)),
  (1.0, (235, 255, 245)),
];
const AMBER: Stops = &[
  (0.0, (0, 0, 0)),
  (0.35, (60, 35, 5)),
  (0.60, (160, 95, 15)),
  (0.82, (235, 170, 40)),
  (1.0, (255, 240, 190)),
];
const COPPER: Stops = &[
  (0.0, (0, 0, 0)),
  (0.30, (45, 18, 6)),
  (0.55, (140, 60, 20)),
  (0.78, (220, 130, 60)),
  (0.92, (250, 200, 130)),
  (1.0, (255, 245, 215)),
];
const SUNSET: Stops = &[
  (0.0, (8, 0, 25)),
  (0.25, (90, 20, 70)),
  (0.50, (200, 60, 80)),
  (0.72, (245, 130, 60)),
  (0.88, (250, 200, 90)),
  (1.0, (255, 240, 200)),
];
const ROSE: Stops = &[
  (0.0, (12, 4, 18)),
  (0.30, (90, 20, 60)),
  (0.55, (190, 50, 110)),
  (0.78, (245, 120, 160)),
  (1.0, (255, 230, 235)),
];
const PLASMA: Stops = &[
  (0.0, (12, 8, 70)),
  (0.30, (90, 10, 140)),
  (0.55, (190, 40, 120)),
  (0.78, (240, 110, 60)),
  (0.92, (250, 200, 70)),
  (1.0, (250, 255, 170)),
];
const OCEAN: Stops = &[
  (0.0, (0, 0, 0)),
  (0.30, (4, 30, 55)),
  (0.55, (10, 90, 110)),
  (0.78, (40, 170, 175)),
  (0.92, (160, 230, 225)),
  (1.0, (240, 255, 255)),
];
const SPECTRUM: Stops = &[
  (0.0, (10, 0, 30)),
  (0.20, (60, 0, 160)),
  (0.40, (0, 130, 220)),
  (0.55, (0, 200, 120)),
  (0.70, (220, 220, 0)),
  (0.85, (240, 110, 20)),
  (1.0, (250, 60, 60)),
];
const INFRARED: Stops = &[
  (0.0, (0, 0, 0)),
  (0.25, (25, 0, 45)),
  (0.48, (115, 0, 80)),
  (0.68, (220, 25, 65)),
  (0.86, (255, 125, 55)),
  (1.0, (255, 240, 180)),
];
const TOXIC: Stops = &[
  (0.0, (0, 0, 0)),
  (0.24, (10, 30, 20)),
  (0.45, (25, 95, 35)),
  (0.66, (90, 190, 45)),
  (0.84, (190, 245, 70)),
  (1.0, (245, 255, 190)),
];
const BATHYMETRY: Stops = &[
  (0.0, (0, 0, 8)),
  (0.25, (0, 18, 55)),
  (0.48, (0, 70, 105)),
  (0.68, (0, 145, 145)),
  (0.86, (85, 220, 205)),
  (1.0, (230, 255, 245)),
];
const GEOLOGIC: Stops = &[
  (0.0, (0, 0, 0)),
  (0.26, (30, 24, 20)),
  (0.46, (85, 70, 42)),
  (0.65, (145, 115, 58)),
  (0.82, (205, 175, 92)),
  (1.0, (245, 235, 185)),
];
const STELLAR: Stops = &[
  (0.0, (0, 0, 8)),
  (0.22, (18, 12, 55)),
  (0.42, (55, 40, 140)),
  (0.62, (120, 75, 220)),
  (0.80, (230, 135, 90)),
  (1.0, (255, 250, 210)),
];
const DUSK: Stops = &[
  (0.0, (8, 4, 18)),
  (0.25, (45, 24, 50)),
  (0.47, (105, 58, 65)),
  (0.67, (175, 105, 70)),
  (0.84, (230, 170, 100)),
  (1.0, (255, 235, 180)),
];
const XRAY: Stops = &[
  (0.0, (0, 0, 0)),
  (0.30, (8, 18, 28)),
  (0.52, (28, 70, 95)),
  (0.72, (95, 165, 185)),
  (0.88, (180, 230, 230)),
  (1.0, (245, 255, 255)),
];

/// `t` key in the HUD cycles through this order.
pub const THEME_CYCLE: &[&str] = &[
  "grayscale",
  "scene",
  "mono",
  "fire",
  "lava",
  "ice",
  "nebula",
  "aurora",
  "amber",
  "copper",
  "sunset",
  "rose",
  "plasma",
  "ocean",
  "spectrum",
  "infrared",
  "toxic",
  "bathymetry",
  "geologic",
  "stellar",
  "dusk",
  "xray",
];

pub const STEPS: usize = 256;

fn build(stops: Stops) -> Vec<(u8, u8, u8)> {
  let mut table = Vec::with_capacity(STEPS);
  for i in 0..STEPS {
    let pos = i as f64 / (STEPS - 1) as f64;
    let mut lo = stops[0];
    let mut hi = *stops.last().unwrap();
    for w in stops.windows(2) {
      if w[0].0 <= pos && pos <= w[1].0 {
        lo = w[0];
        hi = w[1];
        break;
      }
    }
    let span = hi.0 - lo.0;
    let f = if span <= 0.0 { 0.0 } else { (pos - lo.0) / span };
    let r = (lo.1 .0 as f64 + (hi.1 .0 as f64 - lo.1 .0 as f64) * f).round() as u8;
    let g = (lo.1 .1 as f64 + (hi.1 .1 as f64 - lo.1 .1 as f64) * f).round() as u8;
    let b = (lo.1 .2 as f64 + (hi.1 .2 as f64 - lo.1 .2 as f64) * f).round() as u8;
    table.push((r, g, b));
  }
  table
}

// One OnceLock per theme. Each holds a Vec<(u8,u8,u8)> of length STEPS.
// First access for a given theme runs `build` once; thereafter `palette_table`
// is a match + an `as_slice()` -- no locking, no allocation.
macro_rules! palette_slot {
  ($name:ident) => {{
    static T: OnceLock<Vec<(u8, u8, u8)>> = OnceLock::new();
    &T
  }};
}

pub fn palette_table(name: &str) -> &'static [(u8, u8, u8)] {
  let (slot, stops): (&'static OnceLock<Vec<(u8, u8, u8)>>, Stops) = match name {
    "mono" => (palette_slot!(mono), MONO),
    "fire" => (palette_slot!(fire), FIRE),
    "lava" => (palette_slot!(lava), LAVA),
    "ice" => (palette_slot!(ice), ICE),
    "nebula" => (palette_slot!(nebula), NEBULA),
    "aurora" => (palette_slot!(aurora), AURORA),
    "amber" => (palette_slot!(amber), AMBER),
    "copper" => (palette_slot!(copper), COPPER),
    "sunset" => (palette_slot!(sunset), SUNSET),
    "rose" => (palette_slot!(rose), ROSE),
    "plasma" => (palette_slot!(plasma), PLASMA),
    "ocean" => (palette_slot!(ocean), OCEAN),
    "spectrum" => (palette_slot!(spectrum), SPECTRUM),
    "infrared" => (palette_slot!(infrared), INFRARED),
    "toxic" => (palette_slot!(toxic), TOXIC),
    "bathymetry" => (palette_slot!(bathymetry), BATHYMETRY),
    "geologic" => (palette_slot!(geologic), GEOLOGIC),
    "stellar" => (palette_slot!(stellar), STELLAR),
    "dusk" => (palette_slot!(dusk), DUSK),
    "xray" => (palette_slot!(xray), XRAY),
    _ => (palette_slot!(mono), MONO),
  };
  slot.get_or_init(|| build(stops)).as_slice()
}
