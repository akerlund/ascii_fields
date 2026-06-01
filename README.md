# ascii-fields

Procedural ASCII animations for the terminal, now as a native Rust binary.

`ascii-fields` renders animated scenes with ANSI escape codes: surf, solar
flares, galaxies and black holes, quantum clouds, fractals, Game of Life,
physics demos, DNA, molecules, reaction-diffusion, Chladni plates, curl-noise
flow, DLA frost, the Lorenz attractor, drum eigenmodes, and more. There are 71
modes in all.

## Requirements

- Rust / Cargo
- A terminal with ANSI 256-color support
- Truecolor support for named color themes
- A monospace font

Install Rust if needed:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"
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
cargo run --release -- plasma --seconds 5
```

`--release` matters. Debug builds are much slower for the per-cell math.

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

## Options

Common examples:

```bash
./target/release/ascii-fields galaxy --width 120 --height 36
./target/release/ascii-fields waves --fps 18
./target/release/ascii-fields flares --seconds 10
./target/release/ascii-fields black-hole --contrast 1.2
./target/release/ascii-fields areas --scale 0.8
./target/release/ascii-fields night-sky --brightness 1.3
```

CLI options:

```text
--list
--favorites
--width <WIDTH>
--height <HEIGHT>
--fps <FPS>
--seconds <SECONDS>
--scale <SCALE>
--contrast <CONTRAST>
--brightness <BRIGHTNESS>
--speed <SPEED>
--theme <THEME>
--no-status
--blocks
--charset <CHARSET>
--scroll
--export-gif <PATH>
--export-asciinema <PATH>
```

Use `--favorites` to restrict mode listing and playlists to scenes you saved
with `f`:

```bash
./target/release/ascii-fields --list --favorites
./target/release/ascii-fields random --favorites
./target/release/ascii-fields cycle --favorites --seconds 6
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

```text
grayscale, scene, mono, fire, lava, ice, nebula, aurora, amber,
copper, sunset, rose, plasma, ocean, spectrum, infrared, toxic,
bathymetry, geologic, stellar, dusk, xray
```

## Wave-Plane Character Modes

`wave-plane` renders with the `clean` character ramp by default. Switch ramps
or use the smooth block look:

```bash
./target/release/ascii-fields wave-plane
./target/release/ascii-fields wave-plane --charset dense
./target/release/ascii-fields wave-plane --scroll
./target/release/ascii-fields wave-plane --blocks
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
| `+` `-` | Motion speed up / down |
| `s` | Save current per-mode settings to `ascii_fields.json` |
| `f` | Add the current animation to favorites in `ascii_fields.json` |
| `q` / `Esc` | Quit |

Edits to `scale`, `contrast`, `brightness`, `speed`, and `theme` stick to the
scene you make them on. Switch with `n`/`p`, tune another scene, switch back,
and the first scene keeps its values.

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
      "speed": 1.0
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
`render_field()`, `render_glyph_field()`, or `render_block_field()`. The shared
renderer handles character mapping, palette lookup, and ANSI batching.

## Development

```bash
cargo check
cargo test
cargo build --release
./target/release/ascii-fields galaxy --width 60 --height 20 --seconds 1
```

Quick export smoke test:

```bash
./target/release/ascii-fields plasma --width 32 --height 12 --fps 2 --seconds 1 --export-gif /tmp/ascii-fields.gif --export-asciinema /tmp/ascii-fields.cast
```
