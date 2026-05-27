# ascii_fields

Procedural grayscale ASCII animations for the terminal.

`ascii_fields` renders animated terminal scenes using plain Python and ANSI escape codes. It started as a cyclic wave-plane experiment and grew into a small collection of terminal fields: surf, solar flares, galaxies, black holes, and dark spherical ASCII textures.

## Requirements

- Python 3.10+
- A terminal with ANSI color support
- A monospace font

No external Python packages are required.

## Usage

Run from the repo root:

```bash
./ascii_fields.py
```

Stop an animation with `Ctrl+C`.

List modes and presets:

```bash
./ascii_fields.py --list
```

Run a mode:

```bash
./ascii_fields.py wave-plane
./ascii_fields.py areas
./ascii_fields.py waves
./ascii_fields.py flares
./ascii_fields.py galaxy
./ascii_fields.py black-hole
```

You can also use `--mode`:

```bash
./ascii_fields.py --mode galaxy
```

Common options:

```bash
./ascii_fields.py galaxy --width 120 --height 36
./ascii_fields.py waves --fps 18
./ascii_fields.py flares --seconds 10
./ascii_fields.py black-hole --contrast 1.2
./ascii_fields.py areas --scale 0.8
```

## Presets

Each mode has presets. Use `--list` to see them.

```bash
./ascii_fields.py waves --preset calm
./ascii_fields.py flares --preset intense
./ascii_fields.py galaxy --preset wide
./ascii_fields.py black-hole --preset bright
```

## Modes

Base wave plane:

```bash
./ascii_fields.py wave-plane
./ascii_fields.py wave-plane --ascii
./ascii_fields.py wave-plane --ascii --charset clean
./ascii_fields.py wave-plane --ascii --charset dense
```

Dark sphere modes:

```bash
./ascii_fields.py flower-sphere
./ascii_fields.py areas
```

Natural and space modes:

```bash
./ascii_fields.py waves
./ascii_fields.py flares
./ascii_fields.py galaxy
./ascii_fields.py black-hole
```

Aliases are available for a few old names: `codex` maps to `flower-sphere`, and `blackhole` maps to `black-hole`.

`--period` controls the looping base modes. The newer scene modes (`areas`, `waves`, `flares`, `galaxy`, `black-hole`) use elapsed time so they continue evolving without a visible loop reset.

## Project Layout

```text
ascii_fields.py                # CLI entrypoint
cli.py                         # argument parsing and mode selection
core.py                        # terminal runner, shared options, helpers
registry.py                    # animation registry and aliases
presets.py                     # mode-specific presets
animations/
  areas.py
  black_hole.py
  codex.py                     # FlowerSphereAnimation, kept in codex.py for history
  flares.py
  galaxy.py
  plane.py                     # WavePlaneAnimation
  waves.py
tests/
  test_smoke.py
```

Each animation is implemented as a class with a `render()` method. Shared terminal behavior lives in `TerminalRunner`.

## Development

Syntax-check everything:

```bash
python3 -m compileall -q .
```

Run smoke tests:

```bash
python3 -m unittest discover -s tests
```

Quick visual smoke test:

```bash
./ascii_fields.py galaxy --width 60 --height 20 --seconds 1
```

## Capture Ideas

The repo will be easier to understand with terminal captures. Good next artifacts would be:

- `docs/screenshots/galaxy.png`
- `docs/screenshots/black-hole.png`
- `docs/gifs/waves.gif`
- `docs/gifs/areas.gif`

Tools like `asciinema`, `vhs`, or a terminal GIF recorder can make those captures later.
