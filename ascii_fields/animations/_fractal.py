"""Shared helpers for the fractal scenes.

NumPy is used when available (much faster); otherwise a pure-Python escape-time
loop runs with a modest iteration cap so it stays usable in a terminal.
"""

import math

try:
  import numpy as _np
  HAS_NUMPY = True
except ImportError:  # pragma: no cover
  _np = None
  HAS_NUMPY = False

LOG2 = math.log(2.0)


def zoom_scale(elapsed, base, depth, speed):
  """Endless in-and-out zoom: oscillates the magnification forever."""
  logz = depth * 0.5 * (1.0 - math.cos(elapsed * speed))
  return base * math.exp(-logz), logz


def axes(width, height, cx, cy, scale):
  aspect = width / max(1, height * 2.0)
  xs = [cx + (col / max(1, width - 1) - 0.5) * 2.0 * scale * aspect for col in range(width)]
  ys = [cy + (row / max(1, height - 1) - 0.5) * 2.0 * scale for row in range(height)]
  return xs, ys


def _smooth(i, zr2, zi2, max_iter):
  mag2 = zr2 + zi2
  if mag2 <= 1.0:
    return i / max_iter
  nu = i + 1 - math.log(0.5 * math.log(mag2)) / LOG2
  return max(0.0, min(1.0, nu / max_iter))


def mandelbrot_grid(width, height, cx, cy, scale, max_iter, ship=False):
  if HAS_NUMPY:
    return _escape_np(width, height, cx, cy, scale, max_iter, julia_c=None, ship=ship)
  xs, ys = axes(width, height, cx, cy, scale)
  grid = []
  for ci in ys:
    row = []
    for cr in xs:
      zr = zi = 0.0
      level = 0.0
      for i in range(max_iter):
        if ship:
          zr = abs(zr)
          zi = abs(zi)
        zr2 = zr * zr
        zi2 = zi * zi
        if zr2 + zi2 > 16.0:
          level = _smooth(i, zr2, zi2, max_iter)
          break
        zi = 2.0 * zr * zi + ci
        zr = zr2 - zi2 + cr
      row.append(level)
    grid.append(row)
  return grid


def julia_grid(width, height, cx, cy, scale, cr_c, ci_c, max_iter):
  if HAS_NUMPY:
    return _escape_np(width, height, cx, cy, scale, max_iter, julia_c=(cr_c, ci_c))
  xs, ys = axes(width, height, cx, cy, scale)
  grid = []
  for zi0 in ys:
    row = []
    for zr0 in xs:
      zr, zi = zr0, zi0
      level = 0.0
      for i in range(max_iter):
        zr2 = zr * zr
        zi2 = zi * zi
        if zr2 + zi2 > 16.0:
          level = _smooth(i, zr2, zi2, max_iter)
          break
        zi = 2.0 * zr * zi + ci_c
        zr = zr2 - zi2 + cr_c
      row.append(level)
    grid.append(row)
  return grid


def _escape_np(width, height, cx, cy, scale, max_iter, julia_c=None, ship=False):
  aspect = width / max(1, height * 2.0)
  re = _np.linspace(cx - scale * aspect, cx + scale * aspect, width)
  im = _np.linspace(cy - scale, cy + scale, height)
  RE, IM = _np.meshgrid(re, im)
  if julia_c is None:
    C = RE + 1j * IM
    Z = _np.zeros_like(C)
  else:
    C = _np.full(RE.shape, julia_c[0] + 1j * julia_c[1])
    Z = RE + 1j * IM
  out = _np.zeros(RE.shape)
  alive = _np.ones(RE.shape, dtype=bool)
  for i in range(max_iter):
    if ship:
      Z = _np.abs(Z.real) + 1j * _np.abs(Z.imag)
    Z = Z * Z + C
    mag2 = Z.real * Z.real + Z.imag * Z.imag
    escaped = alive & (mag2 > 16.0)
    nu = i + 1 - _np.log(0.5 * _np.log(_np.maximum(mag2, 1.0001))) / LOG2
    out[escaped] = nu[escaped] / max_iter
    alive &= ~escaped
    if not alive.any():
      break
  _np.clip(out, 0.0, 1.0, out=out)
  return out.tolist()


def newton_grid(width, height, cx, cy, scale, angle, max_iter):
  # roots of z^3 = 1, rotated by `angle` so basins swirl over time
  roots = [complex(math.cos(angle + k * 2.0 * math.pi / 3.0),
                   math.sin(angle + k * 2.0 * math.pi / 3.0)) for k in range(3)]
  if HAS_NUMPY:
    aspect = width / max(1, height * 2.0)
    re = _np.linspace(cx - scale * aspect, cx + scale * aspect, width)
    im = _np.linspace(cy - scale, cy + scale, height)
    RE, IM = _np.meshgrid(re, im)
    Z = RE + 1j * IM
    target = _np.exp(1j * 3.0 * angle)
    for _ in range(max_iter):
      Z2 = Z * Z
      Z = Z - (Z2 * Z - target) / (3.0 * Z2 + 1e-9)
    out = _np.zeros(RE.shape)
    for k, root in enumerate(roots):
      shade = 0.30 + 0.30 * k
      near = _np.abs(Z - root) < 0.1
      mag = _np.abs(Z - root)
      out[near] = shade + 0.30 * _np.exp(-mag[near] * 6.0)
    return out.tolist()
  xs, ys = axes(width, height, cx, cy, scale)
  target = complex(math.cos(3.0 * angle), math.sin(3.0 * angle))
  grid = []
  for y in ys:
    row = []
    for x in xs:
      z = complex(x, y)
      for _ in range(max_iter):
        z2 = z * z
        z = z - (z2 * z - target) / (3.0 * z2 + 1e-9)
      level = 0.0
      for k, root in enumerate(roots):
        if abs(z - root) < 0.1:
          level = 0.30 + 0.30 * k + 0.30 * math.exp(-abs(z - root) * 6.0)
          break
      row.append(level)
    grid.append(row)
  return grid


def iteration_cap(width, height, lo=60, hi=110):
  cells = width * max(1, height)
  if HAS_NUMPY:
    return hi
  budget = int(700000 / cells) + 40
  return max(lo, min(hi, budget))
