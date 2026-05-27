PRESETS = {
  "wave-plane": {
    "default": {},
    "dense": {"scale": 2.0, "contrast": 1.15},
    "slow": {"period": 60.0, "fps": 18.0},
  },
  "flower-sphere": {
    "default": {},
    "soft": {"contrast": 0.85},
    "bright": {"contrast": 1.25},
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
    "bright": {"contrast": 1.3},
  },
  "black-hole": {
    "default": {},
    "soft": {"scale": 0.8, "contrast": 0.95},
    "bright": {"contrast": 1.25},
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
