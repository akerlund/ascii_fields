import re
import contextlib
import io
import json
import tempfile
import unittest

from ascii_fields.core import RenderOptions
from ascii_fields.registry import ANIMATIONS, create_animation, normalize_mode
from ascii_fields.presets import preset_names, resolve_preset
from ascii_fields.cli import build_options, build_provider, load_saved_options, parse_args
from ascii_fields.runner import TerminalRunner

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
    for theme in ("grayscale", "scene", "fire", "lava", "copper", "sunset", "rose", "spectrum"):
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
    self.assertEqual(normalize_mode("pcb"), "circuit")
    self.assertEqual(normalize_mode("topology"), "network")
    self.assertEqual(normalize_mode("jupiter"), "storm")
    self.assertEqual(normalize_mode("storm"), "storm")
    self.assertEqual(normalize_mode("magma"), "lava")
    self.assertEqual(normalize_mode("lava-lamp"), "vax_lamp")
    self.assertEqual(normalize_mode("pipeline"), "cpu")

  def test_auto_theme_is_not_accepted(self):
    with contextlib.redirect_stderr(io.StringIO()), self.assertRaises(SystemExit):
      parse_args(["plasma", "--theme", "auto"])

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


class SettingsTests(unittest.TestCase):
  def test_missing_settings_file_uses_defaults(self):
    with tempfile.TemporaryDirectory() as tmp:
      saved = load_saved_options(f"{tmp}/missing.json")
    self.assertEqual(saved, {})
    options = build_options(parse_args(["plasma"]), {}, {})
    self.assertEqual(options.theme, "grayscale")
    self.assertEqual(options.scale, 1.0)
    self.assertEqual(options.speed, 1.0)
    saved_options = build_options(parse_args(["plasma"]), {}, {"speed": 1.75})
    self.assertEqual(saved_options.speed, 1.75)

  def test_saved_options_are_one_dict_per_mode(self):
    with tempfile.TemporaryDirectory() as tmp:
      path = f"{tmp}/ascii_fields.json"
      mode_options = {
        "dna": RenderOptions(theme="scene", scale=1.2, contrast=1.3, brightness=0.9, speed=1.4),
        "mach": RenderOptions(theme="grayscale", scale=1.0, contrast=1.1, brightness=1.0, speed=0.8),
      }
      runner = TerminalRunner(build_provider("dna", parse_args(["dna"])),
                              mode_options=mode_options, settings_path=path)
      runner._save_options()
      with open(path, "r", encoding="utf-8") as handle:
        data = json.load(handle)
    self.assertEqual(set(data), {"dna", "mach"})
    self.assertEqual(data["dna"]["theme"], "scene")
    self.assertEqual(data["dna"]["speed"], 1.4)
    self.assertNotIn("frame", data["dna"])


if __name__ == "__main__":
  unittest.main()
