# ascii-fields

Procedural ASCII animations for the terminal, now as a native Rust binary.

`ascii-fields` renders animated scenes with ANSI escape codes: surf, solar
flares, galaxies and black holes, quantum clouds, fractals, Game of Life,
physics demos, DNA, molecules, reaction-diffusion, Chladni plates, curl-noise
flow, DLA frost, the Lorenz attractor, drum eigenmodes, and more. There are 71
modes in all.

## Requirements

- Rust **1.80 or newer** (1.75 also works if you pin `rayon` to `1.10` and
  `rayon-core` to `1.12.1` in `Cargo.lock`)
- Cargo (ships with the rustup toolchain)
- A terminal with ANSI 256-color support
- Truecolor support for named color themes
- A monospace font

### Installing Rust

The recommended installer is [`rustup`](https://rustup.rs/). It installs
`rustc`, `cargo`, and lets you add components like `rustfmt` and `clippy`
without needing root:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"
```

Verify and pick up the latest stable plus the formatter / linter components:

```bash
rustc --version            # should print 1.80.0 or newer
rustup update stable
rustup component add rustfmt clippy
```

If you want the optional nightly extras documented in [`rustfmt.toml`](rustfmt.toml)
(struct-field column alignment, grouped imports), also install the nightly
toolchain with its own `rustfmt`:

```bash
rustup toolchain install nightly --component rustfmt
```

### Already Have A Distro `rustc`?

Distro packages (apt / dnf / pacman) ship `rustc` and `cargo` but typically
**not** `rustfmt`, which is why `cargo fmt` fails with `no such command: 'fmt'`
on a fresh Ubuntu / Debian install.

Two ways to resolve it:

```bash
# Quick: install rustfmt alongside the existing distro rustc.
# Stable-only -- no `cargo +nightly fmt`.
sudo apt install rustfmt

# Better long term: swap apt rustc for rustup. Newer toolchain, nightly
# support, easier component management.
sudo apt remove rustc cargo
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"
rustup component add rustfmt clippy
```

## Build And Run

```bash
cargo build --release
./target/release/ascii-fields --list
./target/release/ascii-fields plasma
./target/release/ascii-fields random
```

Or build and run in one command:

```bash
cargo run --release -- galaxy
cargo run --release -- magnetic --theme scene
cargo run --release -- random --seconds 5
```

## Options

Common examples:

```bash
./target/release/ascii-fields galaxy --width 120 --height 36
./target/release/ascii-fields waves --fps 18
./target/release/ascii-fields flares --theme scene
./target/release/ascii-fields black-hole --contrast 1.2
./target/release/ascii-fields areas --scale 0.8
./target/release/ascii-fields night-sky --brightness 1.3
./target/release/ascii-fields wave-plane --charset dense
```

CLI options:

| Name | Value | Description |
| --- | --- | --- |
| `--list` | | Print available modes and playlists, then exit |
| `--favorites` | | Restrict listing, `random`, `cycle`, and scene switching to saved favorites |
| `--width` | `<WIDTH>` | Render at this many terminal columns, capped by the current terminal size |
| `--height` | `<HEIGHT>` | Render at this many terminal rows, with HUD rows reserved in interactive mode |
| `--fps` | `<FPS>` | Desired frame rate; defaults to `24` |
| `--seconds` | `<SECONDS>` | Per-scene duration for interactive playlists, or total duration for exports |
| `--scale` | `<SCALE>` | Scene-specific scale or density multiplier |
| `--contrast` | `<CONTRAST>` | Multiply contrast before mapping brightness to glyphs/colors |
| `--brightness` | `<BRIGHTNESS>` | Multiply final brightness |
| `--speed` | `<SPEED>` | Animation time multiplier |
| `--theme` | `<THEME>` | Force a color theme, or use `scene` for each mode's default palette |
| `--no-status` | | Start with the HUD hidden |
| `--charset` | `<scene\|clean\|soft\|dense\|minimal\|blocks>` | Override density-field glyphs, or use `scene` for each mode's built-in glyph style |
| `--scroll` | | Pan the `wave-plane` pattern diagonally; other modes ignore it |
| `--export-gif` | `<PATH>` | Render an animated GIF and exit |
| `--export-asciinema` | `<PATH>` | Render an asciinema v2 `.cast` file and exit |

How `--seconds` works:

| Run Type | Meaning |
| --- | --- |
| Interactive `random` / `cycle` | Time each scene stays active before advancing |
| Export mode | Total exported clip duration |
| Single interactive mode | Ignored; runs until `q` / `Esc` |

```bash
./target/release/ascii-fields random --seconds 4
./target/release/ascii-fields cycle --seconds 15
./target/release/ascii-fields plasma --seconds 5 --export-gif plasma.gif
```

Use `--favorites` to restrict mode listing and playlists to scenes you saved
with `f`:

```bash
./target/release/ascii-fields --list --favorites
./target/release/ascii-fields random --favorites
./target/release/ascii-fields cycle --favorites --seconds 6
```

## Modes

List everything:

```bash
./target/release/ascii-fields --list
```

### Classic Fields

| Mode | What you will see |
| --- | --- |
| `wave-plane` | Cyclic grayscale wave plane with a long character ramp |
| `flower-sphere` | Dark sphere with a rotating flower-like ASCII texture |
| `areas` | Dark oval sphere with floating regions of ASCII texture |

### Nature And Space

| Mode | What you will see |
| --- | --- |
| `waves` | Top-down surf breaking on a beach |
| `flares` | Solar flare-like arcs and tendrils |
| `galaxy` | Rotating spiral galaxy |
| `black-hole` | Lensed black hole accretion disk |
| `night-sky` | Twinkling stars, milky way, and shooting stars |
| `aurora` | Northern lights curtains |
| `clouds` | Drifting fractal-noise clouds |
| `pulsar` | Rotating neutron-star beam sweep |
| `supernova` | Expanding stellar shock shell and filaments |
| `solar-wind` | Charged particles flowing around a magnetosphere |
| `whirlpool` | Swirling vortex / maelstrom |
| `drops` | Raindrops rippling across a pond |
| `caustics` | Underwater light caustics rippling over a surface |

### Life, Light, And Weather

| Mode | What you will see |
| --- | --- |
| `life` | Conway's Game of Life with fading trails |
| `rain` | Falling Matrix-style character rain |
| `fire` | Rising campfire flames |
| `lightning` | Branching lightning bolts in a storm |
| `plasma` | Classic sinusoidal plasma |
| `starfield` | Flying through a 3D starfield |
| `lightspeed` | Jump to lightspeed star streaks |

### Quantum

| Mode | What you will see |
| --- | --- |
| `orbitals` | Hydrogen electron probability clouds |
| `qfield` | Quantum vacuum foam with particle pairs |
| `double-slit` | Double-slit interference build-up |
| `wave-well` | Quantum wave packet in a potential well |

### Physics And Engineering

| Mode | What you will see |
| --- | --- |
| `gwaves` | Black holes merging, gravitational waves |
| `lensing` | Gravitational lensing: stars bent by a moving mass |
| `doppler` | Exoplanet radial-velocity Doppler shift |
| `storm` | Jupiter Great Red Spot vortex |
| `lava` | Boiling lava with bursting bubbles |
| `vax_lamp` | Wax lamp blobs stretching and merging |
| `circuit` | Circuit board traces with data pulses |
| `network` | Network topology with moving packets |
| `cpu` | CPU pipeline, registers, ALU, cache and data pulses |
| `mach` | Sonic boom / Mach cone from a moving source |
| `magnetic` | Bar-magnet dipole field lines |
| `ferrofluid` | Magnetic fluid spikes around moving field sources |
| `schlieren` | Heat-haze and shockwave density gradients |
| `seismograph` | Earthquake wavefronts through layered ground |
| `convection` | Rayleigh-Benard-like heat convection rolls |
| `reconnection` | Magnetic field lines snapping and reconnecting |
| `longitudinal` | Longitudinal compression wave |
| `feynman` | Animated Feynman diagrams |
| `chladni` | Chladni plate nodal patterns |
| `drum` | Vibrational eigenmodes of a circular drum |
| `karman` | Karman vortex street behind a circular obstacle |
| `topography` | Animated contour map with rivers and flow lines |

### Biology And Chemistry

| Mode | What you will see |
| --- | --- |
| `dna` | Rotating DNA double helix |
| `molecule` | Rotating 3D molecular cage |
| `rd` | Gray-Scott reaction-diffusion |

### Generative And Chaos

| Mode | What you will see |
| --- | --- |
| `curl` | Particles drifting through a curl-noise flow field |
| `dla` | Diffusion-limited aggregation |
| `dunes` | Wind-driven sand ripples migrating over dunes |
| `phyllotaxis` | Golden-angle sunflower spiral |
| `quasicrystal` | Fivefold wave interference quasicrystal |
| `moire` | Rotating line-grid moire interference |
| `penrose` | Aperiodic Penrose-like interference tiling |
| `voronoi` | Moving Voronoi cell boundaries |
| `reaction-rings` | Belousov-Zhabotinsky-style chemical wave rings |
| `nbody` | Gravitational bodies orbiting through a shared field |
| `strange` | Morphing De Jong strange attractor |
| `lorenz` | The Lorenz strange attractor |

### Fractals

| Mode | What you will see |
| --- | --- |
| `mandelbrot` | Endless zoom into the Mandelbrot set |
| `julia` | Morphing, zooming Julia set |
| `burning-ship` | Endless zoom into the Burning Ship fractal |
| `newton` | Newton's-method fractal with rotating roots |
| `sierpinski` | Zooming Sierpinski triangle fractal |

### Geometry

| Mode | What you will see |
| --- | --- |
| `hypercube` | Rotating 4D tesseract wireframe |
| `tunnel` | Flight through a winding tunnel |

### Playlists

| Mode | What you will see |
| --- | --- |
| `random` | Shuffle through every animation |
| `cycle` | Step through every animation in order |

`random` and `cycle` show each scene for 10 seconds by default. In interactive
playlist runs, `--seconds` controls how long each scene stays active before
advancing:

```bash
./target/release/ascii-fields random --seconds 4
./target/release/ascii-fields cycle --seconds 15
```

## Color Themes

By default, everything renders in grayscale. Use `--theme scene` to let each
mode choose its own color palette, or force a specific palette:

```bash
./target/release/ascii-fields fire --theme scene
./target/release/ascii-fields aurora --theme scene
./target/release/ascii-fields galaxy --theme nebula
./target/release/ascii-fields plasma --theme spectrum
./target/release/ascii-fields mandelbrot --theme copper
```

Themes:

| Theme | Description |
| --- | --- |
| `grayscale` | Classic monochrome ramp using terminal grayscale |
| `scene` | Use the animation's built-in default palette |
| `mono` | Truecolor black-to-white ramp |
| `fire` | Black, deep red, orange, yellow, and white-hot highlights |
| `lava` | Dark red magma tones with hot orange/yellow peaks |
| `ice` | Dark blue through cyan into icy white |
| `nebula` | Black, violet, magenta, pink, and pale star-cloud highlights |
| `aurora` | Dark green through teal and mint-white |
| `amber` | Warm brown/gold terminal glow |
| `copper` | Dark copper through orange metal highlights |
| `sunset` | Purple shadow, rose, orange, and warm yellow |
| `rose` | Dark plum through pink and pale rose |
| `plasma` | Purple/blue into magenta, orange, yellow, and pale green |
| `ocean` | Deep blue, teal, aqua, and sea-foam white |
| `spectrum` | Wide rainbow-like sweep from violet through blue/green/yellow to red |
| `infrared` | Thermal-camera feel: black, violet, magenta, red, orange, pale yellow |
| `toxic` | Dark green through acid yellow-green |
| `bathymetry` | Deep water blue through cyan and pale green-white |
| `geologic` | Dark rock, olive-brown, ochre, and pale sediment tones |
| `stellar` | Deep space blue/violet with hot orange and pale star highlights |
| `dusk` | Muted violet, mauve, clay-orange, and warm dusk highlights |
| `xray` | Black, blue-gray, cyan-white, and bright diagnostic highlights |

## Charset And Motion

`--charset` changes the brightness-to-character mapping, not the color theme.
The default is `scene`, which lets each animation use its own built-in glyph
style. Density-field animations can use the shared charset overrides; glyph and
diagram animations such as `cpu`, `circuit`, `network`, `dna`, `rain`,
`feynman`, `nbody`, and `doppler` stay locked to `scene`.

Character/block modes:

| Charset | Look | Best For |
| --- | --- | --- |
| `scene` | The animation's own glyph ramp | Default look; best when a mode has a hand-tuned character style |
| `clean` | Long, detailed ASCII ramp | Smooth text shading with lots of intermediate texture |
| `soft` | Shorter, lighter ramp | Less visual noise, easier to read in small terminals |
| `dense` | Compact high-contrast ramp | Stronger texture and darker wave bands |
| `minimal` | Tiny ramp with only a few characters | Posterized, chunky shapes with less fine detail |
| `blocks` | Solid colored terminal cells instead of text glyphs | Smooth color-field look |

Unknown charset values fall back to `scene`.

Motion option:

| Option | Look | Best For |
| --- | --- | --- |
| `--scroll` | Pans the `wave-plane` pattern diagonally | Adds visible drift while keeping the selected charset |

```bash
./target/release/ascii-fields wave-plane
./target/release/ascii-fields plasma --charset minimal
./target/release/ascii-fields mandelbrot --charset dense
./target/release/ascii-fields wave-plane --charset clean
./target/release/ascii-fields wave-plane --charset blocks
./target/release/ascii-fields wave-plane --scroll
```

## Interactive Controls

The HUD is pinned to the bottom of the terminal and shows the current scene,
live-editable parameters, and CPU usage. Toggle it with `i`, or start hidden:

```bash
./target/release/ascii-fields random --no-status
```

| Key | Action |
| --- | --- |
| `i` | Show / hide the HUD |
| `space` | Pause / resume |
| `n` / `p` / Tab | Next / previous scene |
| `t` / `T` | Cycle color theme forward / backward |
| `1` `2` | Scale down / up |
| `3` `4` | Contrast down / up |
| `5` `6` | Brightness down / up |
| `c` | Cycle charset: `scene`, `clean`, `soft`, `dense`, `minimal`, `blocks`; locked modes stay on `scene` |
| `+` `-` | Motion speed up / down |
| `s` | Save the current animation's settings to `ascii_fields.json` |
| `f` | Add the current animation to favorites in `ascii_fields.json` |
| `q` / `Esc` | Quit |

Edits to `scale`, `contrast`, `brightness`, `speed`, `theme`, and `charset` are
live for the current scene. Press `s` to keep them. If you switch scenes without
saving, returning to the scene reloads the last saved settings or built-in
defaults. Pressing `f` saves the favorite list only; it does not commit live
setting edits.

## Saving Settings

Pressing `s` or `f` writes one JSON file in the working directory:

```text
ascii_fields.json
```

The file stores favorites plus per-mode settings:

```json
{
  "favorites": ["dna", "caustics"],
  "modes": {
    "dna": {
      "theme": "scene",
      "scale": 1.2,
      "contrast": 1.3,
      "brightness": 1.0,
      "speed": 1.0,
      "charset": "dense"
    }
  }
}
```

If `ascii_fields.json` is missing or unreadable, built-in defaults are used.
Explicit command-line options still override saved values. Older files keyed
directly by mode name are still loaded and will be rewritten to this structure
the next time settings or favorites are saved.

## Exporting

Export mode renders a finite deterministic clip and exits. In export mode,
`--seconds` controls the total exported duration. Use `--fps`, `--width`, and
`--height` to control the capture. If `--seconds` is omitted, exports default
to a 5-second clip.

```bash
./target/release/ascii-fields plasma --width 100 --height 40 --seconds 5 --export-asciinema plasma.cast
./target/release/ascii-fields plasma --width 100 --height 40 --seconds 5 --export-gif plasma.gif
./target/release/ascii-fields random --width 120 --height 45 --fps 20 --seconds 12 --export-gif random.gif
```

Write both formats from one command:

```bash
./target/release/ascii-fields lava --seconds 5 --export-gif lava.gif --export-asciinema lava.cast
```

GIF export rasterizes ANSI-colored terminal frames using a built-in 8x8 bitmap
font, so it does not need system fonts or external converters.

Play asciinema casts with:

```bash
asciinema play plasma.cast
```

## Project Layout

```text
Cargo.toml
Cargo.lock
src/
  main.rs              entrypoint
  cli.rs               clap CLI
  options.rs           RenderOptions
  animation.rs         Animation trait and FrameContext
  core.rs              math helpers and field renderers
  export.rs            GIF and asciinema export
  noise.rs             value noise, fbm, star noise
  themes.rs            gradient palettes
  runner.rs            terminal loop, keys, HUD
  playlist.rs          single / random / cycle scene providers
  settings.rs          ascii_fields.json load/save
  registry.rs          mode-name -> animation factory
  animations/
    fractal_base.rs    shared escape-time helpers
    *.rs               one animation module, or grouped native Rust modes
```

Each animation builds a grid of brightness levels in `[0, 1]` and hands it to
`render_field()` or `render_glyph_field()`. The shared renderer handles
character mapping, block charset rendering, palette lookup, and ANSI batching.

## Development

```bash
cargo check
cargo test
cargo build --release
./target/release/ascii-fields galaxy --width 60 --height 20
```

Quick export smoke test:

```bash
./target/release/ascii-fields plasma --width 32 --height 12 --fps 2 --seconds 1 --export-gif /tmp/ascii-fields.gif --export-asciinema /tmp/ascii-fields.cast
```

### Debug vs Release Builds

Cargo has two built-in profiles. Every example above uses `--release` because
the animations are math-heavy and the difference is the gap between *smooth at
24 FPS* and *visibly stuttering*.

| Profile | Command | Output | Optimisations | When To Use |
| --- | --- | --- | --- | --- |
| `dev` (default) | `cargo build` | `target/debug/ascii-fields` | None (`opt-level = 0`), debug symbols on | Fastest *compile*, slowest *runtime*. Use while editing code so you don't wait 40s for each rebuild. |
| `release` | `cargo build --release` | `target/release/ascii-fields` | `opt-level = 3`, LTO defaults, no debug overhead | Use for running the animations, exporting GIFs, and any FPS work. |

There is no separate "production" or "test" target binary — `cargo test`
compiles the test profile (basically dev + `cfg(test)`) automatically. You can
add your own profiles in `Cargo.toml` (e.g. a `[profile.bench]` block) but for
this project the two built-ins are all you need.

If `cargo build --release` feels too slow during development, run `cargo
check` instead. It type-checks without producing a binary and is roughly an
order of magnitude faster.

### Using rustfmt.toml

The project ships a [`rustfmt.toml`](rustfmt.toml) at the repo root. `cargo
fmt` reads it automatically — there is nothing to wire up. Settings of note:

- `max_width = 110` — wider than the default 100 so the dense math in the
  animation modules does not get awkwardly line-wrapped.
- `use_small_heuristics = "Max"` — keeps short blocks and literals on one
  line aggressively. This is what lets idioms like `let w = ctx.width; let h
  = ctx.height;` survive a format pass.
- `reorder_imports` / `reorder_modules` — alphabetises imports and `mod`
  declarations.
- `newline_style = "Unix"` — pins LF line endings.

Day-to-day workflow:

```bash
cargo fmt              # format every .rs file in place
cargo fmt -- --check   # print a diff instead of writing; exits non-zero if dirty
```

For the optional nightly extras (struct-field column alignment, grouped
imports), uncomment the `Nightly-only options` block at the bottom of
`rustfmt.toml` and run:

```bash
cargo +nightly fmt
```

Stable rustfmt refuses to start if `unstable_features = true` is present, so
keep that block commented out unless you are explicitly on nightly.

**Vertical alignment of `let` assignments.** Rust has no widely-supported
formatter that aligns assignment operators across consecutive `let` bindings —
rustfmt actively normalises that away. For the few blocks where hand-aligned
columns aid readability (typically the trig setup at the top of an
animation), mark the binding with `#[rustfmt::skip]`:

```rust
#[rustfmt::skip]
let (cw, ch, ax, t) = (
    width  as f64 * 0.5,
    height as f64 * 0.5,
    width  as f64 / (height as f64 * 2.0),
    ctx.elapsed * 0.55,
);
```

The header comment inside `rustfmt.toml` documents this so the convention is
discoverable from the file itself.
