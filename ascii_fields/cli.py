import argparse

from .core import RenderOptions
from .playlist import CLIP_SECONDS, Playlist
from .presets import preset_names, resolve_preset
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
  parser.add_argument("--theme", choices=THEME_CHOICES, default="auto",
                      help="Colour theme. 'auto' = grayscale, 'scene' = each mode's "
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


def option_value(args, preset, name, default):
  value = getattr(args, name)
  if value is not None:
    return value
  return preset.get(name, default)


def build_options(args, preset):
  return RenderOptions(
    scale=option_value(args, preset, "scale", 1.0),
    contrast=option_value(args, preset, "contrast", 1.05),
    brightness=option_value(args, preset, "brightness", 1.0),
    charset=args.charset,
    scroll=args.scroll,
    ascii_mode=True,
    blocks=args.blocks,
    theme=args.theme,
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

  options = build_options(args, preset)
  runner = TerminalRunner(
    build_provider(mode, args),
    width=args.width,
    height=args.height,
    fps=option_value(args, preset, "fps", 24.0),
    seconds=args.seconds,
    period=option_value(args, preset, "period", 24.0),
    options=options,
    record_path=args.record,
    gif_path=args.gif,
    interactive=not args.no_status,
  )
  runner.run()
