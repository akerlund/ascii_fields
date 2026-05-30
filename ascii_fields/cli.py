import argparse
import json
import os

from .core import RenderOptions
from .playlist import CLIP_SECONDS, Playlist
from .presets import PRESETS, preset_names, resolve_preset
from .registry import (
  ANIMATIONS,
  MODE_CHOICES,
  MODE_NAMES,
  PLAYLIST_MODES,
  create_animation,
  normalize_mode,
)
from .runner import TerminalRunner
from .themes import THEME_CYCLE

THEME_CHOICES = THEME_CYCLE
SETTINGS_FILE = "ascii_fields.json"


def parse_args(argv=None):
  parser = argparse.ArgumentParser(
    prog="ascii-fields",
    description="Render procedural ASCII animations in the terminal.",
  )
  parser.add_argument(
    "mode",
    nargs="?",
    choices=(*MODE_CHOICES, "list"),
    default=None,
    metavar="MODE",
    help="Animation mode (or 'random'/'cycle'/'list'). See --list.",
  )
  parser.add_argument("--mode", dest="mode_option", choices=MODE_CHOICES, metavar="MODE",
                      help="Animation mode, as an option instead of a positional argument.")
  parser.add_argument("--list", action="store_true", help="List available modes and presets.")
  parser.add_argument("--width", type=int, default=0, help="Render width in cells.")
  parser.add_argument("--height", type=int, default=0, help="Render height in cells.")
  parser.add_argument("--fps", type=float, default=None, help="Target frames per second.")
  parser.add_argument("--seconds", type=float, default=0.0,
                      help="Stop after this many seconds. 0 runs forever.")
  parser.add_argument("--period", type=float, default=None,
                      help="Loop duration in seconds for the looping base modes.")
  parser.add_argument("--scale", type=float, default=None, help="Animation density multiplier.")
  parser.add_argument("--contrast", type=float, default=None, help="Contrast multiplier.")
  parser.add_argument("--brightness", type=float, default=None,
                      help="Brightness multiplier for the grayscale/colour ramp.")
  parser.add_argument("--speed", type=float, default=None,
                      help="Motion speed multiplier used by live playback.")
  parser.add_argument("--theme", choices=THEME_CHOICES, default=None,
                      help="Colour theme. 'grayscale' = mono, 'scene' = each mode's "
                           "recommended colour, or pick one (needs a truecolor terminal).")
  parser.add_argument("--preset", default="default", help="Mode-specific preset.")
  parser.add_argument("--record", default=None, metavar="FILE.cast",
                      help="Record to an asciinema v2 cast file instead of playing live.")
  parser.add_argument("--gif", default=None, metavar="FILE.gif",
                      help="Render to an animated GIF instead of playing live (needs Pillow). "
                           "Use --seconds to set length and --width/--height for size.")
  parser.add_argument("--no-status", action="store_true",
                      help="Hide the bottom status/controls line during live playback.")
  parser.add_argument("--ascii", action="store_true",
                      help="Legacy flag (wave-plane already uses characters by default).")
  parser.add_argument("--blocks", action="store_true",
                      help="Render wave-plane as smooth coloured blocks instead of characters.")
  parser.add_argument("--charset", choices=("clean", "soft", "dense"), default="clean",
                      help="Character density ramp used by wave-plane.")
  parser.add_argument("--scroll", action="store_true",
                      help="Slide the cyclic wave-plane through the viewport (wraps edges).")
  return parser.parse_args(argv)


def print_modes():
  print("Available modes:")
  for name in MODE_NAMES:
    spec = ANIMATIONS[name]
    aliases = f" (aliases: {', '.join(spec.aliases)})" if spec.aliases else ""
    presets = ", ".join(preset_names(name))
    print(f"  {name:<14} {spec.description}{aliases}")
    print(f"  {'':<14} presets: {presets}")
  print()
  print("Playlists:")
  print(f"  {'random':<14} shuffle through every mode, {CLIP_SECONDS:.0f}s each")
  print(f"  {'cycle':<14} step through every mode in order, {CLIP_SECONDS:.0f}s each")
  print()
  print(f"Themes: {', '.join(THEME_CHOICES)}")


def selected_mode(args):
  mode = args.mode_option or args.mode or "wave-plane"
  if mode in ("list", *PLAYLIST_MODES):
    return mode
  return normalize_mode(mode)


def option_value(args, preset, saved, name, default):
  value = getattr(args, name)
  if value is not None:
    return value
  if name in saved:
    return saved[name]
  return preset.get(name, default)


def _saved_options_for(saved, mode):
  data = saved.get(mode, {})
  if not isinstance(data, dict):
    return {}
  allowed = {"scale", "contrast", "brightness", "speed", "theme"}
  return {key: data[key] for key in allowed if key in data}


def load_saved_options(path=SETTINGS_FILE):
  if not os.path.exists(path):
    return {}
  try:
    with open(path, "r", encoding="utf-8") as handle:
      data = json.load(handle)
  except (OSError, json.JSONDecodeError):
    return {}
  return data if isinstance(data, dict) else {}


def build_options(args, preset, saved=None):
  saved = saved or {}
  return RenderOptions(
    scale=option_value(args, preset, saved, "scale", 1.0),
    contrast=option_value(args, preset, saved, "contrast", 1.05),
    brightness=option_value(args, preset, saved, "brightness", 1.0),
    speed=option_value(args, preset, saved, "speed", 1.0),
    charset=args.charset,
    scroll=args.scroll,
    ascii_mode=True,
    blocks=args.blocks,
    theme=option_value(args, preset, saved, "theme", "grayscale"),
  )


def build_provider(mode, args):
  """Always a Playlist over every mode so n/p switches scenes universally."""
  entries = [(name, (lambda n=name: create_animation(n))) for name in MODE_NAMES]
  if mode == "random":
    return Playlist(entries, clip_seconds=CLIP_SECONDS, shuffle=True, auto_advance=True)
  if mode == "cycle":
    return Playlist(entries, clip_seconds=CLIP_SECONDS, shuffle=False, auto_advance=True)
  return Playlist(entries, shuffle=False, auto_advance=False, start_name=mode)


def main(argv=None):
  args = parse_args(argv)
  mode = selected_mode(args)

  if args.list or mode == "list":
    print_modes()
    return

  if mode in PLAYLIST_MODES:
    preset = {}
  else:
    try:
      preset = resolve_preset(mode, args.preset)
    except ValueError as error:
      raise SystemExit(str(error)) from error

  # Build a separate RenderOptions bundle per mode so live edits stick to the
  # scene that owns them; the active bundle is swapped when you press n/p.
  saved_options = load_saved_options()
  mode_options = {
    name: build_options(
      args,
      PRESETS.get(name, {}).get("default", {}),
      _saved_options_for(saved_options, name),
    )
    for name in MODE_NAMES
  }
  if mode in MODE_NAMES:
    mode_options[mode] = build_options(args, preset, _saved_options_for(saved_options, mode))
  options = mode_options.get(mode, build_options(args, preset))

  runner = TerminalRunner(
    build_provider(mode, args),
    width=args.width,
    height=args.height,
    fps=option_value(args, preset, {}, "fps", 24.0),
    seconds=args.seconds,
    period=option_value(args, preset, {}, "period", 24.0),
    options=options,
    mode_options=mode_options,
    settings_path=SETTINGS_FILE,
    record_path=args.record,
    gif_path=args.gif,
    interactive=not args.no_status,
  )
  runner.run()
