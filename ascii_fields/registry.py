"""Animation registry.

Imports are *lazy*: AnimationSpec stores the module + class name as strings,
and animation_cls is a property that resolves them through importlib on first
access. That means ``--list``, tab-completion, and single-mode startup don't
pay the cost of importing the other 52 animation modules. importlib caches
the import in ``sys.modules`` so subsequent reads are free.
"""

import importlib
from dataclasses import dataclass, field


@dataclass(frozen=True)
class AnimationSpec:
  name: str
  module: str                     # module name under ascii_fields.animations
  class_name: str
  description: str
  aliases: tuple[str, ...] = field(default_factory=tuple)

  @property
  def animation_cls(self):
    mod = importlib.import_module(
      f".animations.{self.module}", package="ascii_fields"
    )
    return getattr(mod, self.class_name)


def _s(name, module, class_name, description, *aliases):
  return AnimationSpec(name=name, module=module, class_name=class_name,
                       description=description, aliases=tuple(aliases))


ANIMATIONS = {
  "wave-plane":    _s("wave-plane",    "plane",        "WavePlaneAnimation",          "cyclic grayscale wave plane",                                   "plane", "base"),
  "flower-sphere": _s("flower-sphere", "codex",        "FlowerSphereAnimation",       "dark flower-like ASCII sphere",                                 "codex", "flower"),
  "areas":         _s("areas",         "areas",        "AreasAnimation",              "dark oval sphere with floating ASCII regions"),
  "waves":         _s("waves",         "waves",        "WavesAnimation",              "top-down surf waves hitting a beach"),
  "flares":        _s("flares",        "flares",       "FlaresAnimation",             "solar flare-like arcs and tendrils"),
  "galaxy":        _s("galaxy",        "galaxy",       "GalaxyAnimation",             "rotating spiral galaxy"),
  "black-hole":    _s("black-hole",    "black_hole",   "BlackHoleAnimation",          "lensed black hole accretion disk",                              "blackhole"),
  "night-sky":     _s("night-sky",     "night_sky",    "NightSkyAnimation",           "twinkling stars, milky way, shooting stars",                    "stars", "sky"),
  "life":          _s("life",          "life",         "LifeAnimation",               "Conway's Game of Life with fading trails",                      "gol", "conway"),
  "drops":         _s("drops",         "drops",        "DropsAnimation",              "raindrops rippling across a pond",                              "pond", "ripples"),
  "orbitals":      _s("orbitals",      "orbitals",     "OrbitalsAnimation",           "hydrogen electron probability clouds",                          "hydrogen", "atom"),
  "qfield":        _s("qfield",        "qfield",       "QuantumFieldAnimation",       "quantum vacuum foam with particle pairs",                       "quantum-field", "foam"),
  "double-slit":   _s("double-slit",   "double_slit",  "DoubleSlitAnimation",         "double-slit interference build-up",                             "slit", "interference"),
  "wave-well":     _s("wave-well",     "wave_well",    "WaveWellAnimation",           "quantum wave packet in a potential well",                       "packet", "well"),
  "mandelbrot":    _s("mandelbrot",    "mandelbrot",   "MandelbrotAnimation",         "endless zoom into the Mandelbrot set",                          "mandel"),
  "julia":         _s("julia",         "julia",        "JuliaAnimation",              "morphing, zooming Julia set"),
  "burning-ship":  _s("burning-ship",  "burning_ship", "BurningShipAnimation",        "endless zoom into the Burning Ship fractal",                    "ship"),
  "newton":        _s("newton",        "newton",       "NewtonAnimation",             "Newton's-method fractal with rotating roots"),
  "sierpinski":    _s("sierpinski",    "sierpinski",   "SierpinskiAnimation",         "zooming Sierpinski triangle fractal",                           "triangle", "triangles"),
  "rain":          _s("rain",          "rain",         "RainAnimation",               "falling Matrix-style character rain",                           "matrix"),
  "fire":          _s("fire",          "fire",         "FireAnimation",               "rising campfire flames",                                        "flame"),
  "plasma":        _s("plasma",        "plasma",       "PlasmaAnimation",             "classic sinusoidal plasma"),
  "lightning":     _s("lightning",     "lightning",    "LightningAnimation",          "branching lightning bolts in a storm",                          "bolt"),
  "clouds":        _s("clouds",        "clouds",       "CloudsAnimation",             "drifting fractal-noise clouds"),
  "starfield":     _s("starfield",     "starfield",    "StarfieldAnimation",          "flying through a 3D starfield",                                 "warp"),
  "aurora":        _s("aurora",        "aurora",       "AuroraAnimation",             "northern lights curtains",                                      "borealis"),
  "whirlpool":     _s("whirlpool",     "whirlpool",    "WhirlpoolAnimation",          "swirling vortex / maelstrom",                                   "vortex", "maelstrom"),
  "hypercube":     _s("hypercube",     "hypercube",    "HypercubeAnimation",          "rotating 4D tesseract wireframe",                               "tesseract"),
  "tunnel":        _s("tunnel",        "tunnel",       "TunnelAnimation",             "flight through a winding tunnel",                               "wormhole"),
  "lightspeed":    _s("lightspeed",    "lightspeed",   "LightspeedAnimation",         "jump to lightspeed star streaks",                               "hyperspace", "jump", "falcon"),
  "feynman":       _s("feynman",       "feynman",      "FeynmanAnimation",            "animated Feynman diagrams",                                     "particles"),
  "magnetic":      _s("magnetic",      "magnetic",     "MagneticAnimation",           "bar-magnet dipole field lines",                                 "magnet", "dipole"),
  "gwaves":        _s("gwaves",        "gwaves",       "GravitationalWavesAnimation", "black holes merging, gravitational waves",                      "gravity", "merger", "ligo"),
  "doppler":       _s("doppler",       "doppler",      "DopplerAnimation",            "exoplanet radial-velocity Doppler shift",                       "exoplanet", "redshift"),
  "storm":         _s("storm",         "storm",        "JupiterStormAnimation",       "Jupiter Great Red Spot vortex",                                 "jupiter", "red-spot", "great-red-spot", "eye"),
  "lava":          _s("lava",          "lava",         "LavaAnimation",               "boiling lava with bursting bubbles",                            "magma"),
  "vax_lamp":      _s("vax_lamp",      "vax_lamp",     "VaxLampAnimation",            "wax lamp blobs stretching and merging",                         "wax-lamp", "lava-lamp", "vax"),
  "circuit":       _s("circuit",       "circuit",      "CircuitAnimation",            "circuit board traces with data pulses",                         "pcb", "electronics", "bus"),
  "network":       _s("network",       "network",      "NetworkAnimation",            "network topology with moving packets",                          "topology", "net"),
  "cpu":           _s("cpu",           "cpu",          "CPUAnimation",                "CPU pipeline, registers, ALU, cache and data pulses",           "pipeline", "processor", "alu"),
  "mach":          _s("mach",          "mach",         "MachAnimation",               "sonic boom / Mach cone from a moving source",                   "sonic-boom", "shockwave", "boom"),
  "longitudinal":  _s("longitudinal",  "longitudinal", "LongitudinalAnimation",       "longitudinal compression wave",                                 "compression", "sound"),
  "dna":           _s("dna",           "dna",          "DNAAnimation",                "rotating DNA double helix",                                     "helix"),
  "molecule":      _s("molecule",      "molecule",     "MoleculeAnimation",           "rotating 3D molecular cage",                                    "buckyball", "fullerene"),
  "lensing":       _s("lensing",       "lensing",      "LensingAnimation",            "gravitational lensing: stars bent by a moving mass",            "lens", "einstein", "microlensing"),
  "rd":            _s("rd",            "rd",           "ReactionDiffusionAnimation",  "Gray-Scott reaction-diffusion (spots, stripes, mazes)",         "gray-scott", "react"),
  "chladni":       _s("chladni",       "chladni",      "ChladniAnimation",            "Chladni plate nodal patterns (cymatics)",                       "cymatics", "plate"),
  "curl":          _s("curl",          "curl",         "CurlNoiseAnimation",          "particles drifting through a curl-noise flow field",            "flow", "curl-noise"),
  "dla":           _s("dla",           "dla",          "DLAAnimation",                "diffusion-limited aggregation: a growing fractal frost",        "frost", "aggregation"),
  "karman":        _s("karman",        "karman",       "KarmanAnimation",             "Karman vortex street behind a circular obstacle",               "vortex-street", "wake"),
  "phyllotaxis":   _s("phyllotaxis",   "phyllotaxis",  "PhyllotaxisAnimation",        "golden-angle sunflower spiral",                                 "sunflower", "spiral"),
  "lorenz":        _s("lorenz",        "lorenz",       "LorenzAnimation",             "the Lorenz strange attractor",                                  "attractor", "butterfly"),
  "drum":          _s("drum",          "drum",         "DrumAnimation",               "vibrational eigenmodes of a circular drum (Bessel)",            "bessel", "membrane"),
}

ALIASES = {alias: name for name, spec in ANIMATIONS.items() for alias in spec.aliases}
MODE_NAMES = tuple(ANIMATIONS.keys())
PLAYLIST_MODES = ("random", "cycle")
MODE_CHOICES = tuple(sorted((*ANIMATIONS.keys(), *ALIASES.keys(), *PLAYLIST_MODES)))


def normalize_mode(mode):
  return ALIASES.get(mode, mode)


def create_animation(mode):
  return ANIMATIONS[normalize_mode(mode)].animation_cls()
