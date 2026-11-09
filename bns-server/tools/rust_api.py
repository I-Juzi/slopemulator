#!/usr/bin/env python3
"""Print the public Rust interface of a crate: one line per pub item.
Usage: tools/rust_api.py <crate dir> [name filter]"""
import pathlib, re, sys

root = pathlib.Path(sys.argv[1]) / "src"
filt = sys.argv[2] if len(sys.argv) > 2 else ""
item = re.compile(r"^\s*pub(\([^)]*\))?\s+(async\s+)?(fn|struct|enum|trait|type|const|static|mod)\b")
for f in sorted(root.rglob("*.rs")):
    lines = f.read_text(errors="replace").splitlines()
    for n, line in enumerate(lines, 1):
        if not item.match(line) or filt not in line:
            continue
        sig = line.strip()
        k = n
        while not sig.rstrip().endswith(("{", ";", ")")) and k < len(lines) and k - n < 6:
            sig += " " + lines[k].strip()
            k += 1
        print(f"{f.relative_to(root.parent)}:{n}: {sig.rstrip('{').strip()}")
