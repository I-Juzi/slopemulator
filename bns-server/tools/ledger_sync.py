#!/usr/bin/env python3
"""Copy per-handler spec state from GameDaemon spec files into parity-ledger.tsv.

A handler section is a '## ' heading that contains the command id. Its state is
'done' when the heading says done, else 'partial'. Also copies Rust port state from `/// parity:` tags.
Usage: ledger_sync.py FILE...
"""
import re, sys
from pathlib import Path

LEDGER = Path(__file__).resolve().parents[1] / "Docs/Server/GameDaemon/parity-ledger.tsv"
state = {}
for f in sys.argv[1:]:
    for head in re.findall(r"^## (.+)$", Path(f).read_text(), re.M):
        m = re.match(r"(?:Command )?(\d+)\b", head)
        if m:
            st = "done" if re.search(r"\bdone\b", head, re.I) else "partial"
            state[m.group(1)] = f"{st}:{Path(f).name}"
lines = LEDGER.read_text().splitlines()
cols = lines[0].split("\t"); si = cols.index("spec")
n = 0
for i, line in enumerate(lines[1:], 1):
    row = line.split("\t")
    if row[0] in state and row[2] == "c2s":
        row[si] = state[row[0]]; lines[i] = "\t".join(row); n += 1
# Rust state from `/// parity: <id> <spec>#<section> <ported|partial>` tags.
rust = {}
for src in (LEDGER.parents[3] / "Program/Server/GameDaemon/src").rglob("*.rs"):
    for cid, st in re.findall(r"/// parity: (\d+) \S+ (ported|partial)", src.read_text()):
        rust[cid] = st
ri = cols.index("rust")
for i, line in enumerate(lines[1:], 1):
    row = line.split("\t")
    if row[0] in rust and row[2] == "c2s":
        row[ri] = rust[row[0]]; lines[i] = "\t".join(row)
print(f"rust tags: {len(rust)}")
LEDGER.write_text("\n".join(lines) + "\n")
print(f"updated {n} rows")
