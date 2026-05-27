from dataclasses import dataclass, field

from animations.areas import AreasAnimation
from animations.black_hole import BlackHoleAnimation
from animations.codex import FlowerSphereAnimation
from animations.flares import FlaresAnimation
from animations.galaxy import GalaxyAnimation
from animations.plane import WavePlaneAnimation
from animations.waves import WavesAnimation


@dataclass(frozen=True)
class AnimationSpec:
  name: str
  animation_cls: type
  description: str
  aliases: tuple[str, ...] = field(default_factory=tuple)


ANIMATIONS = {
  "wave-plane": AnimationSpec(
    "wave-plane",
    WavePlaneAnimation,
    "cyclic grayscale wave plane",
    aliases=("plane", "base"),
  ),
  "flower-sphere": AnimationSpec(
    "flower-sphere",
    FlowerSphereAnimation,
    "dark flower-like ASCII sphere",
    aliases=("codex", "flower"),
  ),
  "areas": AnimationSpec(
    "areas",
    AreasAnimation,
    "dark oval sphere with floating ASCII regions",
  ),
  "waves": AnimationSpec(
    "waves",
    WavesAnimation,
    "top-down surf waves hitting a beach",
  ),
  "flares": AnimationSpec(
    "flares",
    FlaresAnimation,
    "solar flare-like arcs and tendrils",
  ),
  "galaxy": AnimationSpec(
    "galaxy",
    GalaxyAnimation,
    "rotating spiral galaxy",
  ),
  "black-hole": AnimationSpec(
    "black-hole",
    BlackHoleAnimation,
    "lensed black hole accretion disk",
    aliases=("blackhole",),
  ),
}

ALIASES = {alias: name for name, spec in ANIMATIONS.items() for alias in spec.aliases}
MODE_NAMES = tuple(ANIMATIONS.keys())
MODE_CHOICES = tuple(sorted((*ANIMATIONS.keys(), *ALIASES.keys())))


def normalize_mode(mode):
  return ALIASES.get(mode, mode)


def create_animation(mode):
  return ANIMATIONS[normalize_mode(mode)].animation_cls()
