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
    Self::new(false, false, CLIP_SECONDS, Some(start_name))
  }
  pub fn random() -> Self { Self::new(true, true, CLIP_SECONDS, None) }
  pub fn cycle() -> Self { Self::new(true, false, CLIP_SECONDS, None) }
  pub fn random_with_clip(clip: f64) -> Self {
    Self::new(true, true, clip.max(0.1), None)
  }
  pub fn cycle_with_clip(clip: f64) -> Self {
    Self::new(true, false, clip.max(0.1), None)
  }

  fn new(auto: bool, shuffle: bool, clip: f64, start_name: Option<&str>) -> Self {
    let entries = registry::MODES;
    let mut order: Vec<usize> = (0..entries.len()).collect();
    let mut rng = Pcg32::new(0xcafef00dd15ea5e5, 0xa02bdbf7bb3c0a7);
    if shuffle { order.shuffle(&mut rng); }
    if let Some(name) = start_name {
      if let Some(target) = entries.iter().position(|m| m.name == name) {
        if let Some(at) = order.iter().position(|&i| i == target) {
          order.swap(0, at);
        }
      }
    }
    let current = (entries[order[0]].factory)();
    Self { entries, order, pos: 0, auto, shuffle, clip, start: 0.0, current, rng }
  }

  pub fn current(&mut self) -> &mut Box<dyn Animation> { &mut self.current }
  pub fn name(&self) -> &'static str { self.entries[self.order[self.pos]].name }
  pub fn title(&self) -> String {
    format!("{}  ({}/{})", self.name(), self.pos + 1, self.entries.len())
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
