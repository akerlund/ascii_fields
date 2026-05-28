from dataclasses import dataclass, field

from .animations.areas import AreasAnimation
from .animations.aurora import AuroraAnimation
from .animations.black_hole import BlackHoleAnimation
from .animations.burning_ship import BurningShipAnimation
from .animations.chladni import ChladniAnimation
from .animations.clouds import CloudsAnimation
from .animations.codex import FlowerSphereAnimation
from .animations.curl import CurlNoiseAnimation
from .animations.dla import DLAAnimation
from .animations.dna import DNAAnimation
from .animations.doppler import DopplerAnimation
from .animations.double_slit import DoubleSlitAnimation
from .animations.drops import DropsAnimation
from .animations.drum import DrumAnimation
from .animations.feynman import FeynmanAnimation
from .animations.fire import FireAnimation
from .animations.gwaves import GravitationalWavesAnimation
from .animations.karman import KarmanAnimation
from .animations.flares import FlaresAnimation
from .animations.galaxy import GalaxyAnimation
from .animations.hypercube import HypercubeAnimation
from .animations.julia import JuliaAnimation
from .animations.lensing import LensingAnimation
from .animations.life import LifeAnimation
from .animations.lightning import LightningAnimation
from .animations.lightspeed import LightspeedAnimation
from .animations.longitudinal import LongitudinalAnimation
from .animations.lorenz import LorenzAnimation
from .animations.mach import MachAnimation
from .animations.magnetic import MagneticAnimation
from .animations.mandelbrot import MandelbrotAnimation
from .animations.molecule import MoleculeAnimation
from .animations.newton import NewtonAnimation
from .animations.night_sky import NightSkyAnimation
from .animations.orbitals import OrbitalsAnimation
from .animations.phyllotaxis import PhyllotaxisAnimation
from .animations.plane import WavePlaneAnimation
from .animations.plasma import PlasmaAnimation
from .animations.qfield import QuantumFieldAnimation
from .animations.rain import RainAnimation
from .animations.rd import ReactionDiffusionAnimation
from .animations.sierpinski import SierpinskiAnimation
from .animations.starfield import StarfieldAnimation
from .animations.tunnel import TunnelAnimation
from .animations.waves import WavesAnimation
from .animations.wave_well import WaveWellAnimation
from .animations.whirlpool import WhirlpoolAnimation


@dataclass(frozen=True)
class AnimationSpec:
  name: str
  animation_cls: type
  description: str
  aliases: tuple[str, ...] = field(default_factory=tuple)


ANIMATIONS = {
  "wave-plane": AnimationSpec(
    "wave-plane", WavePlaneAnimation, "cyclic grayscale wave plane",
    aliases=("plane", "base"),
  ),
  "flower-sphere": AnimationSpec(
    "flower-sphere", FlowerSphereAnimation, "dark flower-like ASCII sphere",
    aliases=("codex", "flower"),
  ),
  "areas": AnimationSpec(
    "areas", AreasAnimation, "dark oval sphere with floating ASCII regions",
  ),
  "waves": AnimationSpec("waves", WavesAnimation, "top-down surf waves hitting a beach"),
  "flares": AnimationSpec("flares", FlaresAnimation, "solar flare-like arcs and tendrils"),
  "galaxy": AnimationSpec("galaxy", GalaxyAnimation, "rotating spiral galaxy"),
  "black-hole": AnimationSpec(
    "black-hole", BlackHoleAnimation, "lensed black hole accretion disk",
    aliases=("blackhole",),
  ),
  "night-sky": AnimationSpec(
    "night-sky", NightSkyAnimation, "twinkling stars, milky way, shooting stars",
    aliases=("stars", "sky"),
  ),
  "life": AnimationSpec(
    "life", LifeAnimation, "Conway's Game of Life with fading trails",
    aliases=("gol", "conway"),
  ),
  "drops": AnimationSpec(
    "drops", DropsAnimation, "raindrops rippling across a pond",
    aliases=("pond", "ripples"),
  ),
  "orbitals": AnimationSpec(
    "orbitals", OrbitalsAnimation, "hydrogen electron probability clouds",
    aliases=("hydrogen", "atom"),
  ),
  "qfield": AnimationSpec(
    "qfield", QuantumFieldAnimation, "quantum vacuum foam with particle pairs",
    aliases=("quantum-field", "foam"),
  ),
  "double-slit": AnimationSpec(
    "double-slit", DoubleSlitAnimation, "double-slit interference build-up",
    aliases=("slit", "interference"),
  ),
  "wave-well": AnimationSpec(
    "wave-well", WaveWellAnimation, "quantum wave packet in a potential well",
    aliases=("packet", "well"),
  ),
  "mandelbrot": AnimationSpec(
    "mandelbrot", MandelbrotAnimation, "endless zoom into the Mandelbrot set",
    aliases=("mandel",),
  ),
  "julia": AnimationSpec("julia", JuliaAnimation, "morphing, zooming Julia set"),
  "burning-ship": AnimationSpec(
    "burning-ship", BurningShipAnimation, "endless zoom into the Burning Ship fractal",
    aliases=("ship",),
  ),
  "newton": AnimationSpec(
    "newton", NewtonAnimation, "Newton's-method fractal with rotating roots",
  ),
  "sierpinski": AnimationSpec(
    "sierpinski", SierpinskiAnimation, "zooming Sierpinski triangle fractal",
    aliases=("triangle", "triangles"),
  ),
  "rain": AnimationSpec(
    "rain", RainAnimation, "falling Matrix-style character rain",
    aliases=("matrix",),
  ),
  "fire": AnimationSpec("fire", FireAnimation, "rising campfire flames", aliases=("flame",)),
  "plasma": AnimationSpec("plasma", PlasmaAnimation, "classic sinusoidal plasma"),
  "lightning": AnimationSpec(
    "lightning", LightningAnimation, "branching lightning bolts in a storm",
    aliases=("storm", "bolt"),
  ),
  "clouds": AnimationSpec("clouds", CloudsAnimation, "drifting fractal-noise clouds"),
  "starfield": AnimationSpec(
    "starfield", StarfieldAnimation, "flying through a 3D starfield",
    aliases=("warp",),
  ),
  "aurora": AnimationSpec(
    "aurora", AuroraAnimation, "northern lights curtains", aliases=("borealis",),
  ),
  "whirlpool": AnimationSpec(
    "whirlpool", WhirlpoolAnimation, "swirling vortex / maelstrom",
    aliases=("vortex", "maelstrom"),
  ),
  "hypercube": AnimationSpec(
    "hypercube", HypercubeAnimation, "rotating 4D tesseract wireframe",
    aliases=("tesseract",),
  ),
  "tunnel": AnimationSpec(
    "tunnel", TunnelAnimation, "flight through a winding tunnel",
    aliases=("wormhole",),
  ),
  "lightspeed": AnimationSpec(
    "lightspeed", LightspeedAnimation, "jump to lightspeed star streaks",
    aliases=("hyperspace", "jump", "falcon"),
  ),
  "feynman": AnimationSpec(
    "feynman", FeynmanAnimation, "animated Feynman diagrams",
    aliases=("particles",),
  ),
  "magnetic": AnimationSpec(
    "magnetic", MagneticAnimation, "bar-magnet dipole field lines",
    aliases=("magnet", "dipole"),
  ),
  "gwaves": AnimationSpec(
    "gwaves", GravitationalWavesAnimation, "black holes merging, gravitational waves",
    aliases=("gravity", "merger", "ligo"),
  ),
  "doppler": AnimationSpec(
    "doppler", DopplerAnimation, "exoplanet radial-velocity Doppler shift",
    aliases=("exoplanet", "redshift"),
  ),
  "mach": AnimationSpec(
    "mach", MachAnimation, "sonic boom / Mach cone from a moving source",
    aliases=("sonic-boom", "shockwave", "boom"),
  ),
  "longitudinal": AnimationSpec(
    "longitudinal", LongitudinalAnimation, "longitudinal compression wave",
    aliases=("compression", "sound"),
  ),
  "dna": AnimationSpec(
    "dna", DNAAnimation, "rotating DNA double helix",
    aliases=("helix",),
  ),
  "molecule": AnimationSpec(
    "molecule", MoleculeAnimation, "rotating 3D molecular cage",
    aliases=("buckyball", "fullerene"),
  ),
  "lensing": AnimationSpec(
    "lensing", LensingAnimation, "gravitational lensing: stars bent by a moving mass",
    aliases=("lens", "einstein", "microlensing"),
  ),
  "rd": AnimationSpec(
    "rd", ReactionDiffusionAnimation, "Gray-Scott reaction-diffusion (spots, stripes, mazes)",
    aliases=("gray-scott", "react"),
  ),
  "chladni": AnimationSpec(
    "chladni", ChladniAnimation, "Chladni plate nodal patterns (cymatics)",
    aliases=("cymatics", "plate"),
  ),
  "curl": AnimationSpec(
    "curl", CurlNoiseAnimation, "particles drifting through a curl-noise flow field",
    aliases=("flow", "curl-noise"),
  ),
  "dla": AnimationSpec(
    "dla", DLAAnimation, "diffusion-limited aggregation: a growing fractal frost",
    aliases=("frost", "aggregation"),
  ),
  "karman": AnimationSpec(
    "karman", KarmanAnimation, "Karman vortex street behind a circular obstacle",
    aliases=("vortex-street", "wake"),
  ),
  "phyllotaxis": AnimationSpec(
    "phyllotaxis", PhyllotaxisAnimation, "golden-angle sunflower spiral",
    aliases=("sunflower", "spiral"),
  ),
  "lorenz": AnimationSpec(
    "lorenz", LorenzAnimation, "the Lorenz strange attractor",
    aliases=("attractor", "butterfly"),
  ),
  "drum": AnimationSpec(
    "drum", DrumAnimation, "vibrational eigenmodes of a circular drum (Bessel)",
    aliases=("bessel", "membrane"),
  ),
}

ALIASES = {alias: name for name, spec in ANIMATIONS.items() for alias in spec.aliases}
MODE_NAMES = tuple(ANIMATIONS.keys())
PLAYLIST_MODES = ("random", "cycle")
MODE_CHOICES = tuple(sorted((*ANIMATIONS.keys(), *ALIASES.keys(), *PLAYLIST_MODES)))


def normalize_mode(mode):
  return ALIASES.get(mode, mode)


def create_animation(mode):
  return ANIMATIONS[normalize_mode(mode)].animation_cls()
