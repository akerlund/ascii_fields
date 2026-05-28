"""Scene provider: a single Playlist class covers single-mode, cycle, and
random. For single-mode runs auto_advance is False and n/p simply walk through
every mode -- so live scene switching works no matter how playback started.
"""

import random


# How long each clip plays in random/cycle mode. Constant for now; --seconds
# stays the total runtime knob.
CLIP_SECONDS = 10.0


class Playlist:
  """A list of (name, factory) entries with optional time-based auto-advance.

  - ``auto_advance=True``  : steps to the next entry every ``clip_seconds``.
  - ``shuffle=True``       : shuffled order (reshuffled each pass).
  - ``start_name``         : start playing this entry first.
  """

  def __init__(self, entries, clip_seconds=CLIP_SECONDS, shuffle=True, seed=None,
               auto_advance=True, start_name=None):
    self._entries = list(entries)
    self._clip = clip_seconds
    self._shuffle = shuffle
    self._auto = auto_advance
    self._rng = random.Random(seed)
    self._order = list(range(len(self._entries)))
    if shuffle:
      self._rng.shuffle(self._order)
    if start_name is not None:
      names = [name for name, _ in self._entries]
      if start_name in names:
        target = names.index(start_name)
        pos = self._order.index(target)
        self._order[0], self._order[pos] = self._order[pos], self._order[0]
    self._pos = 0
    self._start = 0.0
    self._build()

  def _build(self):
    name, factory = self._entries[self._order[self._pos]]
    self._animation = factory()
    self._name = name

  # --- provider interface used by TerminalRunner ----------------------------

  def current(self):
    return self._animation

  def title(self):
    # always show the position so you know where you are in the lineup,
    # even in single-mode runs where n/p still walks every animation
    return f"{self._name}  ({self._pos + 1}/{len(self._entries)})"

  def scene_elapsed(self, virtual):
    return virtual - self._start

  def maybe_advance(self, virtual):
    if not self._auto:
      return False
    if virtual - self._start >= self._clip:
      self._step(+1, virtual)
      return True
    return False

  def go_next(self, virtual):
    self._step(+1, virtual)

  def go_prev(self, virtual):
    self._step(-1, virtual)

  def restart(self, virtual):
    self._start = virtual
    self._build()

  def _step(self, direction, virtual):
    self._pos += direction
    if self._pos >= len(self._order):
      self._pos = 0
      if self._shuffle:
        self._rng.shuffle(self._order)
    elif self._pos < 0:
      self._pos = len(self._order) - 1
    self._start = virtual
    self._build()
