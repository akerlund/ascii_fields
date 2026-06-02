//! ascii-fields binary entry point. All logic lives in the library
//! (`src/lib.rs`) so benches and tests can reuse it.

use ascii_fields::cli;

fn main() {
  if let Err(err) = cli::run() {
    eprintln!("ascii-fields: {err}");
    std::process::exit(1);
  }
}
