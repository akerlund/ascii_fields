import argparse

from core import RenderOptions, TerminalRunner
from presets import preset_names, resolve_preset
from registry import ANIMATIONS, MODE_CHOICES, MODE_NAMES, create_animation, normalize_mode


def parse_args():
  parser = argparse.ArgumentParser(
    description="Render procedural grayscale ASCII animations in the terminal."
  )
  parser.add_argument(
    "mode",
    nargs="?",
    choices=(*MODE_CHOICES, "list"),
    default=None,
    help="Animation mode. Use 'list' to show modes.",
  )
  parser.add_argument(
    "--mode",
    dest="mode_option",
    choices=MODE_CHOICES,
    help="Animation mode, as an option instead of a positional argument.",
  )
  parser.add_argument("--list", action="store_true", help="List available modes and presets.")
  parser.add_argument("--width", type=int, default=0, help="Render width in cells.")
  parser.add_argument("--height", type=int, default=0, help="Render height in cells.")
  parser.add_argument("--fps", type=float, default=None, help="Target frames per second.")
  parser.add_argument("--seconds", type=float, default=0.0, help="Stop after this many seconds. 0 runs forever.")
  parser.add_argument("--period", type=float, default=None, help="Animation loop duration in seconds for looping modes.")
  parser.add_argument("--scale", type=float, default=None, help="Animation density multiplier.")
  parser.add_argument("--contrast", type=float, default=None, help="Grayscale contrast multiplier.")
  parser.add_argument("--preset", default="default", help="Mode-specific preset.")
  parser.add_argument("--ascii", action="store_true", help="Use ASCII characters for wave-plane mode.")
  parser.add_argument(
    "--charset",
    choices=("clean", "soft", "dense"),
    default="soft",
    help="Character density ramp used with --ascii.",
  )
  parser.add_argument(
    "--scroll",
    action="store_true",
    help="Slide the cyclic wave-plane through the viewport. This intentionally wraps edges.",
  )
  return parser.parse_args()


def print_modes():
  print("Available modes:")
  for name in MODE_NAMES:
    spec = ANIMATIONS[name]
    aliases = f" (aliases: {', '.join(spec.aliases)})" if spec.aliases else ""
    presets = ", ".join(preset_names(name))
    print(f"  {name:<14} {spec.description}{aliases}")
    print(f"  {'':<14} presets: {presets}")


def selected_mode(args):
  mode = args.mode_option or args.mode or "wave-plane"
  if mode == "list":
    return mode
  return normalize_mode(mode)


def option_value(args, preset, name, default):
  value = getattr(args, name)
  if value is not None:
    return value
  return preset.get(name, default)


def main():
  args = parse_args()
  mode = selected_mode(args)

  if args.list or mode == "list":
    print_modes()
    return

  try:
    preset = resolve_preset(mode, args.preset)
  except ValueError as error:
    raise SystemExit(str(error)) from error

  options = RenderOptions(
    scale=option_value(args, preset, "scale", 1.0),
    contrast=option_value(args, preset, "contrast", 1.05),
    charset=args.charset,
    scroll=args.scroll,
    ascii_mode=args.ascii,
  )
  runner = TerminalRunner(
    create_animation(mode),
    width=args.width,
    height=args.height,
    fps=option_value(args, preset, "fps", 24.0),
    seconds=args.seconds,
    period=option_value(args, preset, "period", 24.0),
    options=options,
  )
  runner.run()
