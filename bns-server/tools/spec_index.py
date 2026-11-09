#!/usr/bin/env python3
"""Print the headings of one doc with line ranges, or list docs over 40,000 bytes.

Usage: spec_index.py            list large docs with token estimates
       spec_index.py DOC [-3]   headings of DOC (levels 1-2; -3 adds level 3)
"""
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
LIMIT = 40_000

def tokens(p):
    return p.stat().st_size * 100 // 224  # measured 2.24 bytes per token

args = sys.argv[1:]
if not args:
    for doc in sorted(ROOT.glob("Docs/**/*.md"), key=lambda p: -p.stat().st_size):
        if doc.stat().st_size > LIMIT:
            print(f"{tokens(doc):>8,}  {doc.relative_to(ROOT)}")
    sys.exit(0)
doc = Path(args[0])
depth = 3 if "-3" in args else 2
lines = doc.read_text(errors="replace").splitlines()
heads = [(i + 1, l) for i, l in enumerate(lines) if l.startswith("#") and len(l) - len(l.lstrip("#")) <= depth]
for n, (start, head) in enumerate(heads):
    end = heads[n + 1][0] - 1 if n + 1 < len(heads) else len(lines)
    print(f"{start}-{end}\t{head}")
