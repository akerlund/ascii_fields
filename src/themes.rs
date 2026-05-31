//! Truecolor gradient palettes. `palette(name)` returns a 256-entry lookup
//! mapping `level in [0, 1]` to `(r, g, b)`.

use std::sync::OnceLock;

type Stops = &'static [(f64, (u8, u8, u8))];

const MONO: Stops = &[(0.0, (24, 24, 24)), (1.0, (255, 255, 255))];
const FIRE: Stops = &[
  (0.0, (0, 0, 0)), (0.30, (90, 12, 4)), (0.55, (200, 55, 10)),
  (0.78, (255, 150, 25)), (0.92, (255, 225, 110)), (1.0, (255, 255, 230)),
];
const LAVA: Stops = &[
  (0.0, (5, 0, 0)), (0.25, (60, 6, 0)), (0.50, (170, 30, 0)),
  (0.72, (240, 95, 10)), (0.88, (255, 175, 40)), (1.0, (255, 240, 180)),
];
const ICE: Stops = &[
  (0.0, (0, 0, 0)), (0.30, (8, 22, 60)), (0.55, (20, 80, 150)),
  (0.78, (70, 170, 220)), (1.0, (225, 250, 255)),
];
const NEBULA: Stops = &[
  (0.0, (0, 0, 0)), (0.28, (40, 8, 70)), (0.52, (120, 25, 140)),
  (0.74, (210, 60, 160)), (0.90, (250, 150, 200)), (1.0, (255, 240, 250)),
];
const AURORA: Stops = &[
  (0.0, (0, 0, 0)), (0.30, (4, 40, 35)), (0.52, (20, 140, 90)),
  (0.72, (70, 220, 150)), (0.88, (160, 250, 210)), (1.0, (235, 255, 245)),
];
const AMBER: Stops = &[
  (0.0, (0, 0, 0)), (0.35, (60, 35, 5)), (0.60, (160, 95, 15)),
  (0.82, (235, 170, 40)), (1.0, (255, 240, 190)),
];
const COPPER: Stops = &[
  (0.0, (0, 0, 0)), (0.30, (45, 18, 6)), (0.55, (140, 60, 20)),
  (0.78, (220, 130, 60)), (0.92, (250, 200, 130)), (1.0, (255, 245, 215)),
];
const SUNSET: Stops = &[
  (0.0, (8, 0, 25)), (0.25, (90, 20, 70)), (0.50, (200, 60, 80)),
  (0.72, (245, 130, 60)), (0.88, (250, 200, 90)), (1.0, (255, 240, 200)),
];
const ROSE: Stops = &[
  (0.0, (12, 4, 18)), (0.30, (90, 20, 60)), (0.55, (190, 50, 110)),
  (0.78, (245, 120, 160)), (1.0, (255, 230, 235)),
];
const PLASMA: Stops = &[
  (0.0, (12, 8, 70)), (0.30, (90, 10, 140)), (0.55, (190, 40, 120)),
  (0.78, (240, 110, 60)), (0.92, (250, 200, 70)), (1.0, (250, 255, 170)),
];
const OCEAN: Stops = &[
  (0.0, (0, 0, 0)), (0.30, (4, 30, 55)), (0.55, (10, 90, 110)),
  (0.78, (40, 170, 175)), (0.92, (160, 230, 225)), (1.0, (240, 255, 255)),
];
const SPECTRUM: Stops = &[
  (0.0, (10, 0, 30)), (0.20, (60, 0, 160)), (0.40, (0, 130, 220)),
  (0.55, (0, 200, 120)), (0.70, (220, 220, 0)), (0.85, (240, 110, 20)),
  (1.0, (250, 60, 60)),
];

/// `t` key in the HUD cycles through this order.
pub const THEME_CYCLE: &[&str] = &[
  "grayscale", "scene",
  "mono", "fire", "lava", "ice", "nebula", "aurora", "amber",
  "copper", "sunset", "rose", "plasma", "ocean", "spectrum",
];

fn stops_for(name: &str) -> Stops {
  match name {
    "mono" => MONO, "fire" => FIRE, "lava" => LAVA, "ice" => ICE,
    "nebula" => NEBULA, "aurora" => AURORA, "amber" => AMBER,
    "copper" => COPPER, "sunset" => SUNSET, "rose" => ROSE,
    "plasma" => PLASMA, "ocean" => OCEAN, "spectrum" => SPECTRUM,
    _ => MONO,
  }
}

const STEPS: usize = 256;

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

static CACHE: OnceLock<std::sync::Mutex<std::collections::HashMap<String, Vec<(u8, u8, u8)>>>> =
  OnceLock::new();

pub fn palette_sample(name: &str, level: f64) -> (u8, u8, u8) {
  let cache = CACHE.get_or_init(|| std::sync::Mutex::new(Default::default()));
  let mut guard = cache.lock().unwrap();
  let table = guard.entry(name.to_string()).or_insert_with(|| build(stops_for(name)));
  let last = STEPS - 1;
  let mut idx = (level * last as f64 + 0.5) as isize;
  if idx < 0 {
    idx = 0;
  } else if idx as usize > last {
    idx = last as isize;
  }
  table[idx as usize]
}
