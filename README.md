# ascii_fields

Procedural ASCII animations for the terminal.

`ascii_fields` renders animated scenes using plain Python and ANSI escape codes.
It started as a cyclic wave-plane experiment and grew into a collection of
terminal fields: surf, solar flares, galaxies and black holes, plus quantum
clouds, fractals, a Game of Life, a tesseract, a jump to lightspeed, physics
demos (gravitational waves, gravitational lensing, Doppler shift, magnetic
fields, Feynman diagrams), DNA and molecules, reaction-diffusion, Chladni
plates, curl-noise flow, DLA frost, the Lorenz attractor, drum eigenmodes,
and more — 47 modes in all.

## Requirements

- Python 3.10+
- A terminal with 256-colour ANSI support (truecolour for the colour themes)
- A monospace font
- No external Python packages are required to run. `numpy` is optional and only
  speeds up the fractal modes (`pip install ascii-fields[fast]`); `Pillow` is
  optional and only needed for `--gif` export.

## Install / run

Run straight from the repo:

```bash
./ascii_fields.py            # convenience launcher
python3 -m ascii_fields      # module form
```

Or install it (adds an `ascii-fields` command):

```bash
pip install .
ascii-fields galaxy
```

Stop an animation with `Ctrl+C` (or `q`).

## Modes

List everything, with presets and aliases:

```bash
./ascii_fields.py --list
```

### Classic fields

| Mode | What you'll see |
| --- | --- |
| `wave-plane` | the original cyclic grayscale wave plane, rendered with a long character ramp (aliases: `plane`, `base`) |
| `flower-sphere` | a dark sphere with a slowly rotating flower-like ASCII texture (aliases: `codex`, `flower`) |
| `areas` | a dark oval sphere with floating regions of ASCII texture drifting across its face |

### Nature & space

| Mode | What you'll see |
| --- | --- |
| `waves` | top-down view of surf breaking on a beach, with foam, sandbars and backwash |
| `flares` | the limb of a star with arching loops, plumes and corona |
| `galaxy` | a tilted spiral galaxy rotating around a bright core with dust lanes |
| `black-hole` | an accretion disk warped by gravitational lensing around a Schwarzschild hole (alias: `blackhole`) |
| `night-sky` | a still, deep night sky: twinkling stars, the Milky Way band, the odd shooting star (aliases: `stars`, `sky`) |
| `aurora` | shimmering vertical curtains of northern lights with vertical ray structure (alias: `borealis`) |
| `clouds` | drifting fractal-noise cumulus with a brighter horizon glow |
| `whirlpool` | a swirling maelstrom with logarithmic spiral arms and a dark central throat (aliases: `vortex`, `maelstrom`) |
| `drops` | raindrops splashing into a pond, leaving expanding interfering rings (aliases: `pond`, `ripples`) |

### Life, light & weather

| Mode | What you'll see |
| --- | --- |
| `life` | Conway's Game of Life on a torus, with cells leaving fading glow trails (aliases: `gol`, `conway`) |
| `rain` | falling Matrix-style glyph columns, with bright leading head and fading trail (alias: `matrix`) |
| `fire` | a roaring campfire — heat rises into flickering tongues that cool with height (alias: `flame`) |
| `lightning` | a storm sky with forked bolts that strike, illuminate the clouds, then fade (aliases: `storm`, `bolt`) |
| `plasma` | the classic demoscene plasma — layered sines that ripple and breathe |
| `starfield` | flying forward through 3D stars that streak past the viewer (alias: `warp`) |
| `lightspeed` | the Millennium Falcon's jump: stars cruise, stretch, flash, then become hyperspace streaks (aliases: `hyperspace`, `jump`, `falcon`) |

### Quantum

| Mode | What you'll see |
| --- | --- |
| `orbitals` | hydrogen electron probability clouds \|ψ\|² morphing through 1s → 2p → 3d → 4f (aliases: `hydrogen`, `atom`) |
| `qfield` | vacuum fluctuations on a fractal energy field, with particle-antiparticle pairs popping in and annihilating (aliases: `quantum-field`, `foam`) |
| `double-slit` | the textbook experiment: incoming plane wave, two slits, interference fan, and a detector strip that slowly builds up fringes (aliases: `slit`, `interference`) |
| `wave-well` | a Gaussian wave packet sloshing around a 2D potential well, \|ψ\|² spreading and reviving (aliases: `packet`, `well`) |

### Physics & engineering

| Mode | What you'll see |
| --- | --- |
| `gwaves` | two black holes spiral together emitting quadrupole gravitational waves, chirp, merge-flash, and ring down — looping (aliases: `gravity`, `merger`, `ligo`) |
| `lensing` | an invisible mass drifts behind a star field, bending light into arcs and Einstein rings (aliases: `lens`, `einstein`, `microlensing`) |
| `doppler` | an exoplanet tugs its star in a small wobble; the emitted wavefronts bunch ahead (blueshift) and stretch behind (redshift), in colour with `--theme scene` (aliases: `exoplanet`, `redshift`) |
| `mach` | an object accelerates from subsonic to supersonic, piling its wavefronts into a sonic-boom / Mach cone (aliases: `sonic-boom`, `shockwave`, `boom`) |
| `magnetic` | the dipole field of a bar magnet, with field lines arcing from N to S and tracers flowing along them (aliases: `magnet`, `dipole`) |
| `longitudinal` | a lattice of particles bunching into compressions and rarefactions as a sound wave passes (aliases: `compression`, `sound`) |
| `feynman` | animated Feynman diagrams: arrowed fermions, wavy photons, coiled gluons, vertices and labels — cycling between annihilation, Compton scattering and gluon exchange (alias: `particles`) |
| `chladni` | a square plate driven through its eigenmodes; sand piles along the nodal lines of `sin(mπx)sin(nπy) − sin(nπx)sin(mπy)` (aliases: `cymatics`, `plate`) |
| `drum` | the first vibrational eigenmodes of a circular drum, with real Bessel-J radial profiles and `n` angular nodes (aliases: `bessel`, `membrane`) |
| `karman` | fluid streaming past a circular obstacle sheds a Karman vortex street of alternating swirls (aliases: `vortex-street`, `wake`) |

### Biology & chemistry

| Mode | What you'll see |
| --- | --- |
| `dna` | a rotating DNA double helix, two sinusoidal backbones with base-pair rungs (alias: `helix`) |
| `molecule` | a fullerene-like 3D molecular cage of atoms and bonds, tumbling in 3D (aliases: `buckyball`, `fullerene`) |
| `rd` | Gray-Scott reaction-diffusion: organic spots, stripes and mazes that crawl and morph forever (aliases: `gray-scott`, `react`) |

### Generative / chaos

| Mode | What you'll see |
| --- | --- |
| `curl` | many tiny particles drifting through a divergence-free curl-of-noise flow field, leaving fading trails (aliases: `flow`, `curl-noise`) |
| `dla` | diffusion-limited aggregation — random walkers stick when they touch the cluster, growing a branched fractal frost (aliases: `frost`, `aggregation`) |
| `phyllotaxis` | the golden-angle sunflower spiral packing the screen with rotating seed dots (aliases: `sunflower`, `spiral`) |
| `lorenz` | the Lorenz strange attractor traced live, the current point bright with a fading trail (aliases: `attractor`, `butterfly`) |

### Fractals

| Mode | What you'll see |
| --- | --- |
| `mandelbrot` | endless zoom in and back out of the Mandelbrot set near the seahorse valley (alias: `mandel`) |
| `julia` | a Julia set whose constant `c` orbits a circle, continuously morphing the shape |
| `burning-ship` | endless zoom into the jagged, flame-like Burning Ship fractal (alias: `ship`) |
| `newton` | Newton's-method basins for `z³ = 1` with rotating roots — swirling boundaries |
| `sierpinski` | the right-triangle Sierpinski gasket zooming in seamlessly forever (one octave loops perfectly) (aliases: `triangle`, `triangles`) |

### Geometry

| Mode | What you'll see |
| --- | --- |
| `hypercube` | a tesseract (4D hypercube) rotating through planes that don't exist in 3D (alias: `tesseract`) |
| `tunnel` | flying through a winding tunnel — the vanishing point drifts so the bore curves (alias: `wormhole`) |

### Playlists

| Mode | What you'll see |
| --- | --- |
| `random` | shuffle through every animation, 10s each, reshuffled on each full pass |
| `cycle` | walk every animation in order, 10s each |

```bash
./ascii_fields.py galaxy
./ascii_fields.py orbitals
./ascii_fields.py mandelbrot
./ascii_fields.py drops
./ascii_fields.py random        # shuffle through every mode, 10s each
./ascii_fields.py cycle         # step through every mode in order
```

`random` and `cycle` show each scene for a fixed 10s clip (a constant in
`playlist.py`; `--seconds` still controls total runtime).

## Options

```bash
./ascii_fields.py galaxy --width 120 --height 36
./ascii_fields.py waves --fps 18
./ascii_fields.py flares --seconds 10        # stop after 10s (0 = forever)
./ascii_fields.py black-hole --contrast 1.2
./ascii_fields.py areas --scale 0.8
./ascii_fields.py night-sky --brightness 1.3
```

`--period` controls the looping base modes (`wave-plane`, `flower-sphere`). The
scene modes use elapsed time so they evolve without a visible loop reset.

### Presets

Each mode has presets; see `--list`.

```bash
./ascii_fields.py waves --preset calm
./ascii_fields.py fire --preset blaze
./ascii_fields.py galaxy --preset bright
```

### Colour themes

By default everything renders in grayscale. `--theme scene` colours each mode
with a hand-picked palette; or force a specific theme (needs a truecolour
terminal):

```bash
./ascii_fields.py fire --theme scene       # orange flames
./ascii_fields.py aurora --theme scene     # green curtains
./ascii_fields.py galaxy --theme nebula
./ascii_fields.py plasma --theme spectrum
```

Themes: `auto` (grayscale, default), `scene`, `mono`, `fire`, `ice`, `nebula`,
`aurora`, `amber`, `plasma`, `ocean`, `spectrum`.

### Wave-plane character modes

`wave-plane` now renders with the long `clean` character ramp by default, so
you see a rich gradient of glyphs. Switch ramps or fall back to the smooth
coloured-block look:

```bash
./ascii_fields.py wave-plane
./ascii_fields.py wave-plane --charset dense
./ascii_fields.py wave-plane --scroll
./ascii_fields.py wave-plane --blocks       # the old smooth-block style
```

## Interactive controls

A two-line HUD shows the current scene and every live-editable parameter. Toggle
it with `i`. **`n`/`p` walks every animation in single-mode runs too**, so you
can start on one scene and step through the rest without restarting.

| Key | Action |
| --- | --- |
| `i` | show / hide the HUD |
| `space` | pause / resume |
| `n` / `p` / Tab | next / previous scene (works in every mode) |
| `t` / `T` | cycle colour theme forward / backward |
| `,` `.` | scale down / up |
| `;` `'` | contrast down / up |
| `[` `]` | brightness down / up |
| `+` `-` | motion speed up / down |
| `s` | save current frame to `ascii_fields-<timestamp>.ans` |
| `r` | restart current scene |
| `q` / `Esc` | quit |

Start with the HUD hidden via `--no-status`.

## Recording & GIFs

Export an animated **GIF** directly (needs `pip install pillow`). `--seconds`
sets the length, `--width`/`--height` the size:

```bash
./ascii_fields.py galaxy --gif galaxy.gif --seconds 8 --width 100 --height 30
./ascii_fields.py fire --theme scene --gif fire.gif --seconds 6
./ascii_fields.py doppler --theme scene --gif doppler.gif
```

Or record to an [asciinema](https://asciinema.org) v2 cast (no dependencies):

```bash
./ascii_fields.py galaxy --record galaxy.cast --seconds 12
asciinema play galaxy.cast
```

## Project layout

```text
ascii_fields.py                # convenience launcher (./ascii_fields.py)
pyproject.toml                 # packaging; `ascii-fields` console script
ascii_fields/
  cli.py                       # argument parsing and mode selection
  core.py                      # render options, math helpers, render_field()
  runner.py                    # terminal loop: alt-screen, keys, recording
  playlist.py                  # single / random / cycle scene providers
  themes.py                    # truecolour gradient palettes
  noise.py                     # value noise + fbm
  registry.py / presets.py     # mode registry, aliases, presets
  gifexport.py                 # ANSI frames -> animated GIF (Pillow)
  animations/                  # one module per scene (38 of them)
tests/
  test_smoke.py
```

Each animation builds a grid of brightness levels in `[0, 1]` and hands it to
`render_field()` (or `render_glyph_field` / `render_block_field`), which does
the character mapping, palette lookup and run-length ANSI batching once, in one
place.

## Development

```bash
python3 -m compileall -q ascii_fields ascii_fields.py   # syntax check
python3 -m unittest discover -s tests                   # tests
./ascii_fields.py galaxy --width 60 --height 20 --seconds 1   # quick visual
```

## Capture ideas

Use the built-in `--gif` to make artifacts for `docs/` directly:

```bash
./ascii_fields.py cycle --gif docs/gifs/tour.gif --seconds 30
./ascii_fields.py orbitals --gif docs/gifs/orbitals.gif
./ascii_fields.py gwaves --gif docs/gifs/gwaves.gif
```

`--record` (asciinema) is the other route if you prefer casts.
