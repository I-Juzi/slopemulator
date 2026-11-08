import struct, sys
sys.path.insert(0, '.')
from pe import pe
d, w, base, ts, secs = pe('/home/ubuntu/Documents/Codex/private-project/Client.exe')
def off2va(o):
    for n, va, vs, ra, rs, ch, e in secs:
        if ra <= o < ra + rs: return base + va + o - ra
def va2off(v):
    r = v - base
    for n, va, vs, ra, rs, ch, e in secs:
        if va <= r < va + rs: return ra + r - va
for name in ['abnormalcamera', 'account-level', 'record']:
    pat = (name + '\0').encode('utf-16-le'); i = -1; hits = []
    while True:
        i = d.find(pat, i + 1)
        if i < 0 or len(hits) > 5: break
        if i % 2 == 0 and d[i-2:i] in (b'\0\0',) or True: hits.append(i)
    for h in hits[:3]:
        v = off2va(h); refs = []; j = -1; p = struct.pack('<I', v)
        while True:
            j = d.find(p, j + 1)
            if j < 0: break
            refs.append(j)
        print(name, 'str va', hex(v), 'refs', len(refs))
        for r in refs[:3]:
            words = struct.unpack_from('<12I', d, r)
            print('   ref va', hex(off2va(r)), 'sec', [s[0] for s in secs if s[3] <= r < s[3] + s[4]], ' '.join('%08x' % x for x in words))
