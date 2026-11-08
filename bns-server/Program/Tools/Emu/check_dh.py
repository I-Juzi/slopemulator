"""Run only generated DH cases while the RSA vector run completes."""
import json
from emu381 import Emulator
from generate_381 import dh_vectors

emu = Emulator()
cases = dh_vectors(emu)
for case in cases:
    print(json.dumps(case, sort_keys=True))
print(json.dumps({'stub_counts': emu.stub_counts}))
