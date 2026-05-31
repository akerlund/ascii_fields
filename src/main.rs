//! ascii-fields, Rust port.

mod animation;
mod animations;
mod cli;
mod core;
mod export;
mod noise;
mod options;
mod playlist;
mod registry;
mod runner;
mod settings;
mod themes;

fn main() {
  if let Err(err) = cli::run() {
    eprintln!("ascii-fields: {err}");
    std::process::exit(1);
  }
}
