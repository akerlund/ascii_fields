"""Terminal driver: frame loop, alt-screen, resize, keys, and recording."""

import json
import select
import shutil
import sys
import time

from .core import (
  ALT_SCREEN_OFF,
  ALT_SCREEN_ON,
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
               period=24.0, options=None, record_path=None, gif_path=None,
               interactive=True):
    self.provider = provider
    self.width = width
    self.height = height
    self.fps = max(1.0, fps)
    self.seconds = seconds
    self.period = period if period and period > 0 else 24.0
    self.options = options or RenderOptions()
    self.record_path = record_path
    self.gif_path = gif_path
    self.interactive = interactive

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
    speed = 1.0
    paused = False
    last_size = None
    hud_visible = bool(self.interactive)
    self._last_frame = ""

    def step_param(attr, factor, lo=0.1, hi=5.0):
      value = getattr(self.options, attr) * factor
      setattr(self.options, attr, max(lo, min(hi, value)))

    def cycle_theme(direction):
      try:
        idx = THEME_CYCLE.index(self.options.theme)
      except ValueError:
        idx = 0
      self.options.theme = THEME_CYCLE[(idx + direction) % len(THEME_CYCLE)]

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
            virtual += dt * speed

          for key in keyboard.poll():
            if key in ("q", "\x1b", "\x03"):
              return
            elif key == " ":
              paused = not paused
            elif key in ("+", "="):
              speed = min(8.0, speed * 1.25)
            elif key in ("-", "_"):
              speed = max(0.1, speed * 0.8)
            elif key in ("n", "\t"):
              self.provider.go_next(virtual)
            elif key == "p":
              self.provider.go_prev(virtual)
            elif key == "r":
              self.provider.restart(virtual)
            elif key == "s":
              self._save_frame(self._last_frame)
            elif key == "i":
              hud_visible = not hud_visible
              last_size = None        # force a redraw at the new size
            elif key == "t":
              cycle_theme(+1)
            elif key == "T":
              cycle_theme(-1)
            elif key == ",":
              step_param("scale", 0.91)
            elif key == ".":
              step_param("scale", 1.10)
            elif key == ";":
              step_param("contrast", 0.91)
            elif key == "'":
              step_param("contrast", 1.10)
            elif key == "[":
              step_param("brightness", 0.91)
            elif key == "]":
              step_param("brightness", 1.10)

          self.provider.maybe_advance(virtual)
          hud_lines = 2 if hud_visible else 0
          width, height = terminal_size(self.width, self.height, reserve=hud_lines)
          if last_size is not None and (width, height) != last_size:
            out.write(CLEAR)
          last_size = (width, height)

          elapsed = self.provider.scene_elapsed(virtual)
          phase = (elapsed % self.period) / self.period
          frame = self.provider.current().render(width, height, elapsed, phase, self.options)
          self._last_frame = frame
          hud = self._hud(width, hud_visible, paused, speed)
          out.write(HOME + frame + hud)
          out.flush()

          next_frame += frame_time
          time.sleep(max(0.0, next_frame - time.monotonic()))
      except KeyboardInterrupt:
        pass
      finally:
        out.write(RESET + SHOW_CURSOR + ALT_SCREEN_OFF)
        out.flush()

  def _hud(self, width, visible, paused, speed):
    if not visible or not self.interactive or not sys.stdout.isatty():
      return ""
    o = self.options
    title = self.provider.title()
    if paused:
      state = "PAUSED"
    elif abs(speed - 1.0) > 0.02:
      state = f"x{speed:.2f}"
    else:
      state = "play"
    line1 = (f" {title}   theme={o.theme:<8}  scale={o.scale:.2f}  "
             f"contrast={o.contrast:.2f}  bright={o.brightness:.2f}  "
             f"fps={self.fps:.0f}  {state}")
    line2 = (" [i]menu [n/p]switch [t]theme [,/.]scale [;/']ctr [[/]]bright "
             "[_]pause [+/-]speed [s]save [r]reset [q]quit")
    return "\n\x1b[2m" + _fit(line1, width) + "\x1b[0m\n\x1b[2m" + _fit(line2, width) + "\x1b[0m"

  def _save_frame(self, frame):
    if not frame:
      return
    name = f"ascii_fields-{time.strftime('%Y%m%d-%H%M%S')}.ans"
    with open(name, "w", encoding="utf-8") as handle:
      handle.write(frame + RESET + "\n")

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
