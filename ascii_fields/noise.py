"""Deterministic value noise and fractional Brownian motion (no dependencies).

Used by the cloud, aurora, plasma and field scenes that want organic, seamless
texture without random per-frame jitter.
"""

import math


def hash01(ix, iy, seed=0):
  value = (ix * 374761393 + iy * 668265263 + seed * 1442695040888963407) & 0xFFFFFFFF
  value = (value ^ (value >> 13)) * 1274126177 & 0xFFFFFFFF
  value ^= value >> 16
  return (value & 0xFFFF) / 65535.0


def value_noise(x, y, seed=0):
  ix = math.floor(x)
  iy = math.floor(y)
  fx = x - ix
  fy = y - iy
  ux = fx * fx * (3.0 - 2.0 * fx)
  uy = fy * fy * (3.0 - 2.0 * fy)
  a = hash01(ix, iy, seed)
  b = hash01(ix + 1, iy, seed)
  c = hash01(ix, iy + 1, seed)
  d = hash01(ix + 1, iy + 1, seed)
  top = a + (b - a) * ux
  bottom = c + (d - c) * ux
  return top + (bottom - top) * uy


def fbm(x, y, octaves=4, lacunarity=2.0, gain=0.5, seed=0):
  amplitude = 0.5
  frequency = 1.0
  total = 0.0
  norm = 0.0
  for octave in range(octaves):
    total += amplitude * value_noise(x * frequency, y * frequency, seed + octave * 101)
    norm += amplitude
    amplitude *= gain
    frequency *= lacunarity
  return total / norm if norm else 0.0
