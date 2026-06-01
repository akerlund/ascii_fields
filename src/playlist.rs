//! Scene provider: single mode, or a time-advancing playlist (random / cycle).

use rand::seq::SliceRandom;
use rand_pcg::Pcg32;

use crate::animation::Animation;
use crate::registry::{self, ModeInfo};

pub const CLIP_SECONDS: f64 = 10.0;

pub struct Playlist {
  entries: &'static [ModeInfo],
  order: Vec<usize>,
  pos: usize,
  auto: bool,
  shuffle: bool,
  clip: f64,
  start: f64,
  current: Box<dyn Animation>,
  rng: Pcg32,
}

impl Playlist {
  pub fn single(start_name: &str) -> Self {
    Self::new(false, false, CLIP_SECONDS, Some(start_name), None)
      .expect("registry should contain at least one mode")
  }
  pub fn single_from(start_name: &str, names: &[String]) -> Option<Self> {
    Self::new(false, false, CLIP_SECONDS, Some(start_name), Some(names))
  }
  pub fn random() -> Self {
    Self::new(true, true, CLIP_SECONDS, None, None)
      .expect("registry should contain at least one mode")
  }
  pub fn cycle() -> Self {
    Self::new(true, false, CLIP_SECONDS, None, None)
      .expect("registry should contain at least one mode")
  }
  pub fn random_from(names: &[String]) -> Option<Self> {
    Self::new(true, true, CLIP_SECONDS, None, Some(names))
  }
  pub fn cycle_from(names: &[String]) -> Option<Self> {
    Self::new(true, false, CLIP_SECONDS, None, Some(names))
  }
  pub fn random_with_clip(clip: f64) -> Self {
    Self::new(true, true, clip.max(0.1), None, None)
      .expect("registry should contain at least one mode")
  }
  pub fn cycle_with_clip(clip: f64) -> Self {
    Self::new(true, false, clip.max(0.1), None, None)
      .expect("registry should contain at least one mode")
  }
  pub fn random_with_clip_from(clip: f64, names: &[String]) -> Option<Self> {
    Self::new(true, true, clip.max(0.1), None, Some(names))
  }
  pub fn cycle_with_clip_from(clip: f64, names: &[String]) -> Option<Self> {
    Self::new(true, false, clip.max(0.1), None, Some(names))
  }

  fn new(auto: bool, shuffle: bool, clip: f64, start_name: Option<&str>, allowed_names: Option<&[String]>) -> Option<Self> {
    let entries = registry::MODES;
    let mut order: Vec<usize> = match allowed_names {
      Some(names) => {
        let mut out = Vec::new();
        for name in names {
          if let Some(idx) = entries.iter().position(|m| m.name == name.as_str()) {
            if !out.contains(&idx) {
              out.push(idx);
            }
          }
        }
        out
      }
      None => (0..entries.len()).collect(),
    };
    if order.is_empty() {
      return None;
    }
    let mut rng = Pcg32::new(0xcafef00dd15ea5e5, 0xa02bdbf7bb3c0a7);
    if shuffle { order.shuffle(&mut rng); }
    if let Some(name) = start_name {
      let target = entries.iter().position(|m| m.name == name)?;
      if let Some(at) = order.iter().position(|&i| i == target) {
        order.swap(0, at);
      } else {
        return None;
      }
    }
    let current = (entries[order[0]].factory)();
    Some(Self { entries, order, pos: 0, auto, shuffle, clip, start: 0.0, current, rng })
  }

  pub fn current(&mut self) -> &mut Box<dyn Animation> { &mut self.current }
  pub fn name(&self) -> &'static str { self.entries[self.order[self.pos]].name }
  pub fn title(&self) -> String {
    format!("{}  ({}/{})", self.name(), self.pos + 1, self.order.len())
  }
  pub fn scene_elapsed(&self, virtual_time: f64) -> f64 { virtual_time - self.start }

  pub fn maybe_advance(&mut self, virtual_time: f64) -> bool {
    if !self.auto { return false; }
    if virtual_time - self.start >= self.clip {
      self.step(1, virtual_time);
      return true;
    }
    false
  }
  pub fn go_next(&mut self, virtual_time: f64) { self.step(1, virtual_time); }
  pub fn go_prev(&mut self, virtual_time: f64) { self.step(-1, virtual_time); }
  pub fn restart(&mut self, virtual_time: f64) {
    self.start = virtual_time;
    self.current = (self.entries[self.order[self.pos]].factory)();
  }

  fn step(&mut self, direction: i32, virtual_time: f64) {
    let n = self.order.len();
    let mut p = self.pos as i32 + direction;
    if p >= n as i32 {
      p = 0;
      if self.shuffle { self.order.shuffle(&mut self.rng); }
    } else if p < 0 {
      p = n as i32 - 1;
    }
    self.pos = p as usize;
    self.start = virtual_time;
    self.current = (self.entries[self.order[self.pos]].factory)();
  }
}
