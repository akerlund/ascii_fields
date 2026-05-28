#!/usr/bin/env python3
"""Convenience launcher to run from the repo without installing.

Equivalent to ``python -m ascii_fields`` or, after ``pip install .``, the
``ascii-fields`` command.
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from ascii_fields.cli import main

if __name__ == "__main__":
  main()
