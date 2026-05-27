import math
import shutil
import sys
import time
from dataclasses import dataclass


RESET = "\x1b[0m"
HIDE_CURSOR = "\x1b[?25l"
SHOW_CURSOR = "\x1b[?25h"
HOME = "\x1b[H"
CLEAR = "\x1b[2J"
BLACK_BG = "\x1b[48;5;232m"


@dataclass(frozen=True)
class RenderOptions:
  scale: float = 1.0
  contrast: float = 1.05
  charset: str = "soft"
  scroll: bool = False
  ascii_mode: bool = False


class Animation:
  uses_elapsed_time = True

  def render(self, width, height, elapsed, phase, options):
    raise NotImplementedError


def terminal_size(width, height):
  size = shutil.get_terminal_size((100, 40))
  return width or size.columns, height or max(1, size.lines - 1)


def smoothstep(edge0, edge1, value):
  value = max(0.0, min(1.0, (value - edge0) / (edge1 - edge0)))
  return value * value * (3.0 - 2.0 * value)


def shade(value, contrast):
  value = 0.5 + value * 0.42 * contrast
  return max(0.0, min(1.0, value))


def star_noise(ix, iy):
  value = (ix * 374761393 + iy * 668265263) & 0xFFFFFFFF
  value = (value ^ (value >> 13)) * 1274126177
  value = value ^ (value >> 16)
  return (value & 0xFFFF) / 65535.0



def density_char(level, thresholds):
  for limit, char in thresholds:
    if level < limit:
      return char
  return thresholds[-1][1]

def gray_fg(level, base=234, steps=5):
  return base + round(max(0.0, min(1.0, level)) * steps)


class TerminalRunner:
  def __init__(self, animation, width=0, height=0, fps=24.0, seconds=0.0, period=24.0, options=None):
    self.animation = animation
    self.width = width
    self.height = height
    self.fps = fps
    self.seconds = seconds
    self.period = period
    self.options = options or RenderOptions()

  def run(self):
    frame_time = 1.0 / max(1.0, self.fps)
    started = time.monotonic()
    next_frame = started

    sys.stdout.write(HIDE_CURSOR + CLEAR)
    try:
      while True:
        now = time.monotonic()
        elapsed = now - started
        if self.seconds > 0.0 and elapsed >= self.seconds:
          break

        phase = (elapsed % self.period) / self.period
        width, height = terminal_size(self.width, self.height)
        frame = self.animation.render(width, height, elapsed, phase, self.options)
        sys.stdout.write(HOME + frame)
        sys.stdout.flush()

        next_frame += frame_time
        time.sleep(max(0.0, next_frame - time.monotonic()))
    except KeyboardInterrupt:
      pass
    finally:
      sys.stdout.write(RESET + SHOW_CURSOR + "\n")
      sys.stdout.flush()
