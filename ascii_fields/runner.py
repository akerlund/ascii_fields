"""Terminal driver: frame loop, alt-screen, resize, keys, and recording."""

import json
import select
import shutil
import sys
import time

from .core import (
  ALT_SCREEN_OFF,
  ALT_SCREEN_ON,
  BLACK_BG,
  CLEAR,
  HIDE_CURSOR,
  HOME,
  RESET,
  SHOW_CURSOR,
  RenderOptions,
)
from .themes import THEME_CYCLE

try:
  import termios
  import tty
  _HAS_TERMIOS = True
except ImportError:  # pragma: no cover - non-POSIX
  _HAS_TERMIOS = False


def terminal_size(width, height, reserve=1):
  size = shutil.get_terminal_size((100, 40))
  return width or size.columns, height or max(1, size.lines - reserve)


def _fit(text, width):
  """Trim or pad ``text`` to ``width`` columns and emit clear-to-end-of-line."""
  if len(text) >= width:
    return text[:width] + "\x1b[K"
  return text + "\x1b[K"


class _RawInput:
  """Context manager: put the tty in cbreak mode and read keys without blocking."""

  def __init__(self):
    self.enabled = _HAS_TERMIOS and sys.stdin.isatty()
    self.fd = None
    self.saved = None

  def __enter__(self):
    if self.enabled:
      self.fd = sys.stdin.fileno()
      self.saved = termios.tcgetattr(self.fd)
      tty.setcbreak(self.fd)
    return self

  def __exit__(self, *exc):
    if self.enabled and self.saved is not None:
      termios.tcsetattr(self.fd, termios.TCSADRAIN, self.saved)

  def poll(self):
    if not self.enabled:
      return ""
    keys = []
    while select.select([sys.stdin], [], [], 0)[0]:
      keys.append(sys.stdin.read(1))
    return "".join(keys)


class TerminalRunner:
  def __init__(self, provider, width=0, height=0, fps=24.0, seconds=0.0,
               period=24.0, options=None, mode_options=None,
               settings_path="ascii_fields.json",
               record_path=None, gif_path=None, interactive=True):
    self.provider = provider
    self.width = width
    self.height = height
    self.fps = max(1.0, fps)
    self.seconds = seconds
    self.period = period if period and period > 0 else 24.0
    self.options = options or RenderOptions()
    # Each mode keeps its own mutable RenderOptions; switching scenes swaps the
    # active options so live edits stick to that scene and reappear on return.
    self.mode_options = mode_options or {}
    self.settings_path = settings_path
    self.record_path = record_path
    self.gif_path = gif_path
    self.interactive = interactive
    # mutable state used by _hud / _save_options -- safe defaults so callers
    # outside _run_live() (tests, etc.) work too.
    self._last_frame = ""
    self._last_dims = (0, 0)
    self._cpu_pct = 0.0
    self._prev_proc = time.process_time()
    self._prev_wall = time.monotonic()
    self._save_msg = ""
    self._save_until = 0.0

  def run(self):
    if self.gif_path:
      self._run_gif()
    elif self.record_path:
      self._run_recording()
    else:
      self._run_live()

  def _offline_frames(self):
    """Render a fixed-size, fixed-count sequence of frame strings."""
    width, height = terminal_size(self.width, self.height)
    duration = self.seconds if self.seconds > 0.0 else 8.0
    frame_time = 1.0 / self.fps
    count = max(1, int(duration * self.fps))
    frames = []
    virtual = 0.0
    for _ in range(count):
      self.provider.maybe_advance(virtual)
      elapsed = self.provider.scene_elapsed(virtual)
      phase = (elapsed % self.period) / self.period
      frames.append(self.provider.current().render(width, height, elapsed, phase, self.options))
      virtual += frame_time
    return frames, (width, height)

  def _run_gif(self):
    from . import gifexport
    frames, _ = self._offline_frames()
    count, size = gifexport.write_gif(frames, self.gif_path, self.fps)
    print(f"Wrote {count} frames to {self.gif_path} ({size[0]}x{size[1]}px)")

  # -- live playback -------------------------------------------------------

  def _run_live(self):
    frame_time = 1.0 / self.fps
    started = time.monotonic()
    last_wall = started
    next_frame = started
    virtual = 0.0
    paused = False
    last_size = None
    hud_visible = bool(self.interactive)
    current_name = None
    self._last_frame = ""
    self._last_dims = (0, 0)
    self._cpu_pct = 0.0
    self._prev_proc = time.process_time()
    self._prev_wall = started
    self._save_msg = ""
    self._save_until = 0.0

    def step_param(attr, factor, lo=0.1, hi=5.0):
      value = getattr(self.options, attr) * factor
      setattr(self.options, attr, max(lo, min(hi, value)))

    def cycle_theme(direction):
      try:
        idx = THEME_CYCLE.index(self.options.theme)
      except ValueError:
        idx = 0
      self.options.theme = THEME_CYCLE[(idx + direction) % len(THEME_CYCLE)]

    def sync_mode():
      """Swap ``self.options`` to the active mode's stored options bundle."""
      name = getattr(self.provider, "name", lambda: None)()
      if name and name in self.mode_options:
        self.options = self.mode_options[name]
      return name

    out = sys.stdout
    out.write(ALT_SCREEN_ON + HIDE_CURSOR + CLEAR)
    with _RawInput() as keyboard:
      try:
        while True:
          now = time.monotonic()
          if self.seconds > 0.0 and now - started >= self.seconds:
            break

          dt = now - last_wall
          last_wall = now
          if not paused:
            virtual += dt * self.options.speed

          for key in keyboard.poll():
            if key in ("q", "\x1b", "\x03"):
              return
            elif key == " ":
              paused = not paused
            elif key in ("+", "="):
              self.options.speed = min(8.0, self.options.speed * 1.25)
            elif key in ("-", "_"):
              self.options.speed = max(0.1, self.options.speed * 0.8)
            elif key in ("n", "\t"):
              self.provider.go_next(virtual)
            elif key == "p":
              self.provider.go_prev(virtual)
            elif key == "s":
              self._save_options()
            elif key == "i":
              hud_visible = not hud_visible
              last_size = None        # force a redraw at the new size
            elif key == "t":
              cycle_theme(+1)
            elif key == "T":
              cycle_theme(-1)
            elif key == "1":
              step_param("scale", 0.91)
            elif key == "2":
              step_param("scale", 1.10)
            elif key == "3":
              step_param("contrast", 0.91)
            elif key == "4":
              step_param("contrast", 1.10)
            elif key == "5":
              step_param("brightness", 0.91)
            elif key == "6":
              step_param("brightness", 1.10)

          self.provider.maybe_advance(virtual)
          new_name = sync_mode()
          if new_name != current_name:
            current_name = new_name
            last_size = None        # repaint the field at the new mode's options
          term_lines = shutil.get_terminal_size((100, 40)).lines
          hud_lines = 2 if hud_visible else 0
          width, height = terminal_size(self.width, self.height, reserve=hud_lines)
          if last_size is not None and (width, height) != last_size:
            out.write(CLEAR)
          last_size = (width, height)
          self._last_dims = (width, height)

          if not paused:
            elapsed = self.provider.scene_elapsed(virtual)
            phase = (elapsed % self.period) / self.period
            self._last_frame = self.provider.current().render(
              width, height, elapsed, phase, self.options
            )

          # CPU usage: process time vs wall time over the last frame, EMA-smoothed
          now_proc = time.process_time()
          now_wall = time.monotonic()
          d_proc = now_proc - self._prev_proc
          d_wall = now_wall - self._prev_wall
          if d_wall > 0.0:
            instant = (d_proc / d_wall) * 100.0
            self._cpu_pct = 0.80 * self._cpu_pct + 0.20 * instant
          self._prev_proc = now_proc
          self._prev_wall = now_wall

          hud = self._hud(width, hud_visible, paused, term_lines)
          if paused:
            # nothing to redraw on the field -- only the HUD (absolute positioning)
            out.write(hud)
          else:
            out.write(HOME + self._last_frame + hud)
          out.flush()

          next_frame += frame_time
          time.sleep(max(0.0, next_frame - time.monotonic()))
      except KeyboardInterrupt:
        pass
      finally:
        out.write(RESET + SHOW_CURSOR + ALT_SCREEN_OFF)
        out.flush()

  def _hud(self, width, visible, paused, term_lines):
    if not visible or not self.interactive or not sys.stdout.isatty():
      return ""
    o = self.options
    # Fixed-width title so the rest of line 1 doesn't shimmy when the name changes.
    title = f"{self.provider.title():<22}"
    if paused:
      state = "PAUSED"
    elif abs(o.speed - 1.0) > 0.02:
      state = f"x{o.speed:.2f}"
    else:
      state = "play "
    if self._save_msg and time.monotonic() < self._save_until:
      state = self._save_msg
    # Column widths chosen so each control hint in line 2 sits in the SAME
    # column as the matching value in line 1.
    #   title=22 ; theme=14 ; scale=10 ; contrast=13 ; bright=11 ; cpu=10 ; fps=6
    line1 = (
      f" {title}"
      f" theme={o.theme:<10}"                 # 16 chars (label 6 + value 10 fits 'grayscale')
      f" scale={o.scale:>4.2f}"               # 10 chars (label 6 + value 4)
      f" contrast={o.contrast:>4.2f}"         # 13 chars (label 9 + value 4)
      f" bright={o.brightness:>4.2f}"         # 11 chars (label 7 + value 4)
      f" cpu={self._cpu_pct:>5.1f}%"          # 10 chars
      f" fps={self.fps:>2.0f}"                # 6 chars
      f"  {state}"
    )
    # Line 2: [n/p]switch sits at the very left (under the title), then the
    # per-value control hints align with line 1's value columns.
    line2 = (
      f" {'[n/p]switch':<22}"                  # leftmost control, padded to title column
      f" {'[t/T]theme':<16}"                   # under theme=
      f" {'[1/2]scale':<10}"                   # under scale=
      f" {'[3/4]contrast':<13}"                # under contrast=
      f" {'[5/6]bright':<11}"                  # under bright=
      f" {'[i]menu':<10}"
      f"  [_]pause [+/-]speed [s]save [q]quit"
    )
    pos1 = f"\x1b[{max(1, term_lines - 1)};1H"
    pos2 = f"\x1b[{max(1, term_lines)};1H"
    # True black background so the menu reads as a separate black strip from
    # the field's near-black BLACK_BG (which is 232 -> very dark gray).
    style = "\x1b[2m\x1b[40m"
    return (pos1 + style + _fit(line1, width) + "\x1b[0m"
            + pos2 + style + _fit(line2, width) + "\x1b[0m")

  def _save_options(self):
    """Persist one small settings dict per mode to a stable JSON file."""
    options_by_mode = self.mode_options or {}
    if not options_by_mode:
      name = getattr(self.provider, "name", lambda: "current")()
      options_by_mode = {name or "current": self.options}
    data = {}
    for name in sorted(options_by_mode):
      options = options_by_mode[name]
      data[name] = {
        "theme": options.theme,
        "scale": round(options.scale, 4),
        "contrast": round(options.contrast, 4),
        "brightness": round(options.brightness, 4),
        "speed": round(options.speed, 4),
      }
    with open(self.settings_path, "w", encoding="utf-8") as handle:
      json.dump(data, handle, indent=2, sort_keys=True)
      handle.write("\n")
    self._save_msg = f"saved {self.settings_path}"
    self._save_until = time.monotonic() + 2.5

  # -- recording (asciinema v2 cast) --------------------------------------

  def _run_recording(self):
    frames, (width, height) = self._offline_frames()
    frame_time = 1.0 / self.fps
    header = {
      "version": 2,
      "width": width,
      "height": height + 1,
      "timestamp": int(time.time()),
      "env": {"TERM": "xterm-256color"},
    }
    with open(self.record_path, "w", encoding="utf-8") as cast:
      cast.write(json.dumps(header) + "\n")
      cast.write(json.dumps([0.0, "o", HIDE_CURSOR + CLEAR]) + "\n")
      for i, frame in enumerate(frames):
        cast.write(json.dumps([i * frame_time, "o", HOME + frame]) + "\n")
      cast.write(json.dumps([len(frames) * frame_time, "o", RESET + SHOW_CURSOR]) + "\n")
    print(f"Recorded {len(frames)} frames to {self.record_path} "
          f"(play with: asciinema play {self.record_path})")
