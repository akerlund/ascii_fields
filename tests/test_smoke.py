import unittest

from core import RenderOptions
from registry import ANIMATIONS, create_animation, normalize_mode


class RenderSmokeTests(unittest.TestCase):
  def test_every_animation_renders_expected_rows(self):
    options = RenderOptions(ascii_mode=True)
    for mode in ANIMATIONS:
      with self.subTest(mode=mode):
        animation = create_animation(mode)
        frame = animation.render(24, 8, 0.25, 0.25, options)
        rows = frame.split("\n")
        self.assertEqual(len(rows), 8)
        self.assertTrue(all(isinstance(row, str) for row in rows))

  def test_aliases_resolve(self):
    self.assertEqual(normalize_mode("codex"), "flower-sphere")
    self.assertEqual(normalize_mode("blackhole"), "black-hole")


if __name__ == "__main__":
  unittest.main()
