import re
import unittest

from ascii_fields.core import RenderOptions
from ascii_fields.registry import ANIMATIONS, create_animation, normalize_mode
from ascii_fields.presets import preset_names, resolve_preset
from ascii_fields.cli import build_provider, parse_args

ANSI = re.compile(r"\x1b\[[0-9;?]*[a-zA-Z]")


def visible_widths(frame):
  return {len(ANSI.sub("", row)) for row in frame.split("\n")}


class RenderSmokeTests(unittest.TestCase):
  def test_every_animation_renders_expected_shape(self):
    options = RenderOptions()
    for mode in ANIMATIONS:
      with self.subTest(mode=mode):
        animation = create_animation(mode)
        frame = animation.render(24, 8, 0.25, 0.25, options)
        rows = frame.split("\n")
        self.assertEqual(len(rows), 8)

  def test_visible_width_is_consistent(self):
    options = RenderOptions()
    for mode in ANIMATIONS:
      with self.subTest(mode=mode):
        frame = create_animation(mode).render(40, 10, 0.5, 0.3, options)
        self.assertEqual(visible_widths(frame), {40})

  def test_renders_at_degenerate_sizes(self):
    options = RenderOptions()
    for mode in ANIMATIONS:
      for width, height in [(1, 1), (3, 1), (1, 5), (2, 3), (7, 2)]:
        with self.subTest(mode=mode, size=(width, height)):
          frame = create_animation(mode).render(width, height, 0.5, 0.5, options)
          self.assertEqual(len(frame.split("\n")), height)

  def test_themes_render(self):
    for theme in ("auto", "scene", "fire", "spectrum"):
      options = RenderOptions(theme=theme)
      frame = create_animation("plasma").render(20, 6, 0.4, 0.4, options)
      self.assertEqual(len(frame.split("\n")), 6)

  def test_stateful_modes_handle_time_reset(self):
    options = RenderOptions()
    for mode in ("life", "drops", "starfield", "qfield", "gwaves", "doppler", "mach"):
      with self.subTest(mode=mode):
        animation = create_animation(mode)
        animation.render(30, 10, 5.0, 0.5, options)
        # jumping back in time (a scene restart) must not raise
        animation.render(30, 10, 0.1, 0.5, options)

  def test_aliases_resolve(self):
    self.assertEqual(normalize_mode("codex"), "flower-sphere")
    self.assertEqual(normalize_mode("blackhole"), "black-hole")
    self.assertEqual(normalize_mode("hydrogen"), "orbitals")
    self.assertEqual(normalize_mode("tesseract"), "hypercube")

  def test_unknown_preset_raises(self):
    with self.assertRaises(ValueError):
      resolve_preset("galaxy", "does-not-exist")

  def test_every_mode_has_presets(self):
    for mode in ANIMATIONS:
      self.assertIn("default", preset_names(mode))


class PlaylistTests(unittest.TestCase):
  def test_random_and_cycle_build_and_advance(self):
    for mode in ("random", "cycle"):
      provider = build_provider(mode, parse_args([mode]))
      first = provider.title()
      provider.go_next(0.0)
      self.assertNotEqual(first, provider.title())
      self.assertEqual(len(provider.current().render(20, 6, 0.1, 0.1, RenderOptions()).split("\n")), 6)


if __name__ == "__main__":
  unittest.main()
