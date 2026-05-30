PRESETS = {
  "wave-plane": {
    "default": {},
    "dense": {"scale": 2.0, "contrast": 1.15},
    "slow": {"period": 60.0, "fps": 18.0},
  },
  "flower-sphere": {
    "default": {},
    "soft": {"contrast": 0.85},
    "bright": {"contrast": 1.25, "brightness": 1.2},
  },
  "areas": {
    "default": {},
    "wide": {"scale": 0.7, "contrast": 0.95},
    "storm": {"scale": 1.15, "contrast": 1.25},
  },
  "waves": {
    "default": {},
    "calm": {"scale": 0.75, "contrast": 0.9, "fps": 18.0},
    "rough": {"scale": 1.25, "contrast": 1.25},
  },
  "flares": {
    "default": {},
    "soft": {"scale": 0.8, "contrast": 0.9},
    "intense": {"scale": 1.15, "contrast": 1.3},
  },
  "galaxy": {
    "default": {},
    "wide": {"scale": 0.8, "contrast": 1.15},
    "bright": {"contrast": 1.3, "brightness": 1.15},
  },
  "black-hole": {
    "default": {},
    "soft": {"scale": 0.8, "contrast": 0.95},
    "bright": {"contrast": 1.25, "brightness": 1.15},
  },
  "night-sky": {
    "default": {},
    "dense": {"scale": 1.3, "brightness": 1.15},
    "sparse": {"scale": 0.7},
  },
  "life": {
    "default": {},
    "fast": {"fps": 30.0},
  },
  "drops": {
    "default": {},
    "calm": {"contrast": 0.85},
    "downpour": {"contrast": 1.3},
  },
  "orbitals": {
    "default": {},
    "bright": {"contrast": 1.3, "brightness": 1.2},
  },
  "qfield": {
    "default": {},
    "busy": {"contrast": 1.25},
  },
  "double-slit": {
    "default": {},
  },
  "wave-well": {
    "default": {},
    "bright": {"contrast": 1.3, "brightness": 1.2},
  },
  "mandelbrot": {
    "default": {},
    "bright": {"contrast": 1.25, "brightness": 1.2},
  },
  "julia": {
    "default": {},
    "bright": {"contrast": 1.25, "brightness": 1.2},
  },
  "burning-ship": {
    "default": {},
  },
  "newton": {
    "default": {},
  },
  "sierpinski": {
    "default": {},
    "fast": {},
  },
  "rain": {
    "default": {},
    "heavy": {"scale": 1.6},
    "slow": {"scale": 0.6},
  },
  "fire": {
    "default": {},
    "blaze": {"scale": 1.2, "contrast": 1.25, "brightness": 1.15},
  },
  "plasma": {
    "default": {},
    "tight": {"scale": 1.6},
  },
  "lightning": {
    "default": {},
  },
  "clouds": {
    "default": {},
    "thick": {"scale": 0.7, "contrast": 1.2},
  },
  "starfield": {
    "default": {},
    "warp": {"scale": 1.8},
  },
  "aurora": {
    "default": {},
    "bright": {"contrast": 1.3, "brightness": 1.2},
  },
  "whirlpool": {
    "default": {},
    "tight": {"scale": 1.5},
  },
  "hypercube": {
    "default": {},
  },
  "tunnel": {
    "default": {},
    "rings": {"scale": 1.5},
  },
  "lightspeed": {
    "default": {},
  },
  "feynman": {
    "default": {},
  },
  "magnetic": {
    "default": {},
    "dense": {"contrast": 1.25},
  },
  "gwaves": {
    "default": {},
    "bright": {"contrast": 1.3, "brightness": 1.2},
  },
  "doppler": {
    "default": {},
  },
  "storm": {
    "default": {},
    "bright": {"contrast": 1.25, "brightness": 1.1},
  },
  "lava": {
    "default": {},
    "hot": {"contrast": 1.25, "brightness": 1.15},
  },
  "vax_lamp": {
    "default": {},
    "gooey": {"contrast": 1.2, "brightness": 1.1},
  },
  "circuit": {
    "default": {},
    "bright": {"contrast": 1.25, "brightness": 1.15},
  },
  "network": {
    "default": {},
    "bright": {"contrast": 1.25, "brightness": 1.15},
  },
  "cpu": {
    "default": {},
    "bright": {"contrast": 1.25, "brightness": 1.15},
  },
  "mach": {
    "default": {},
  },
  "longitudinal": {
    "default": {},
    "tight": {"scale": 1.8},
  },
  "dna": {
    "default": {},
    "tight": {"scale": 1.6},
  },
  "molecule": {
    "default": {},
  },
  "lensing": {
    "default": {},
    "dense": {"scale": 1.6, "contrast": 1.2},
    "sparse": {"scale": 0.6},
  },
  "rd": {
    "default": {},
    "slow": {"scale": 0.5},
    "fast": {"scale": 1.5},
  },
  "chladni": {
    "default": {},
    "bright": {"contrast": 1.25, "brightness": 1.2},
  },
  "curl": {
    "default": {},
    "wild": {"scale": 1.6},
  },
  "dla": {
    "default": {},
    "fast": {"scale": 1.8},
  },
  "karman": {
    "default": {},
  },
  "phyllotaxis": {
    "default": {},
    "dense": {"scale": 1.8},
    "sparse": {"scale": 0.6},
  },
  "lorenz": {
    "default": {},
    "long-trail": {"scale": 0.6},
  },
  "drum": {
    "default": {},
  },
}


def preset_names(mode):
  return tuple(PRESETS.get(mode, {"default": {}}).keys())


def resolve_preset(mode, preset):
  presets = PRESETS.get(mode, {"default": {}})
  if preset not in presets:
    names = ", ".join(sorted(presets))
    raise ValueError(f"Unknown preset '{preset}' for mode '{mode}'. Available presets: {names}")
  return presets[preset]
