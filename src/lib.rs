//! ascii-fields library surface. Mirrors `main.rs` so benches and
//! external consumers can import the modules; `main.rs` calls into
//! `cli::run` and otherwise stays a thin wrapper.

pub mod animation;
pub mod animations;
pub mod cli;
pub mod core;
pub mod export;
pub mod noise;
pub mod options;
pub mod playlist;
pub mod registry;
pub mod runner;
pub mod settings;
pub mod themes;
