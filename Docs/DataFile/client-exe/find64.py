import struct, sys
sys.path.insert(0, '.')
from pe import pe
d, w, base, ts, secs = pe('/home/ubuntu/Documents/Codex/private-project/BNSR_116.exe')
def off2va(o):
    for n, va, vs, ra, rs, ch, e in secs:
        if ra <= o < ra + rs: return base + va + o - ra
def va2off(v):
    r = v - base
    for n, va, vs, ra, rs, ch, e in secs:
        if va <= r < va + rs: return ra + r - va if r - va < rs else None
def sec(o): return [s[0] for s in secs if s[3] <= o < s[3] + s[4]]
def refs(v):
    p = struct.pack('<Q', v); j = -1; out = []
    while True:
        j = d.find(p, j + 1)
        if j < 0: return out
        out.append(j)
if __name__ == '__main__':
    for name in ['abnormalcamera', 'account-level']:
        pat = (name + '\0').encode('utf-16-le'); i = -1
        while True:
            i = d.find(pat, i + 1)
            if i < 0: break
            v = off2va(i); r = refs(v); print(name, hex(v), sec(i), 'refs', len(r))
            for x in r[:4]: print('   ', hex(off2va(x)), sec(x), struct.unpack_from('<QhhhxxQB', d, x))
