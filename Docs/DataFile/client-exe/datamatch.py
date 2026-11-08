# Match data addresses (globals, vtables) through matched functions, then add vtable-slot function matches.
# usage: python3 -I datamatch.py A.jsonl B.jsonl M_IN.json SRC_4K.pdb TARGET.exe M_OUT.json DATA_OUT.json
import sys, os, json, struct, collections
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import pdbio
fa, fb, min_, srcpdb, tgt, mout, dout = sys.argv[1:8]
def ld(p):
    D = {}
    for l in open(p):
        r = json.loads(l)
        if 'vt' not in r: D[r['ea']] = r
    return D
A = ld(fa); B = ld(fb)
M = json.load(open(min_)); m = {e['a']: e['b'] for e in M}; rm = {b: a for a, b in m.items()}
how = {e['a']: e['how'] for e in M}

# source data symbols from the PDB (VA -> (kind, typind, name)) and data publics
st = pdbio.msf_read(srcpdb); dbi = pdbio.parse_dbi(st[3])
sh = st[dbi['dbg'][5]]
secs = [struct.unpack_from('<II', sh, i*40+8) for i in range(len(sh)//40)]
def va(seg, off): return 0x400000 + secs[seg-1][1] + off
gdata, dpub = {}, {}
for o, k, p in pdbio.iter_syms(st[dbi['symrec']]):
    if k in (0x110d, 0x110c):
        ti, off, seg = struct.unpack_from('<IIH', p, 0)
        if 1 <= seg <= len(secs): gdata.setdefault(va(seg, off), (k, ti, p[10:p.index(b'\0', 10)].decode('latin1')))
    elif k == 0x110e:
        fl, off, seg = struct.unpack_from('<IIH', p, 0)
        if not fl & 2 and 1 <= seg <= len(secs): dpub.setdefault(va(seg, off), p[10:p.index(b'\0', 10)].decode('latin1'))
print('source data symbols', len(gdata), 'data publics', len(dpub), flush=True)

# target image for reading vtables
d = open(tgt, 'rb').read()
pe = struct.unpack_from('<I', d, 0x3c)[0]; n = struct.unpack_from('<H', d, pe+6)[0]; opt = struct.unpack_from('<H', d, pe+20)[0]
tsec = [struct.unpack_from('<8sIIII', d, pe+24+opt+40*i) for i in range(n)]
def rd(v):
    r = v - 0x400000
    for nm, vs, sva, rs, rp in tsec:
        if sva <= r < sva + rs: return struct.unpack_from('<I', d, rp + r - sva)[0]
    return None

for rnd in range(1):
    votes = collections.defaultdict(collections.Counter)
    for a, b in m.items():
        da, db = A[a]['drefs'], B[b]['drefs']
        if len(da) == len(db) and da:
            for x, y in zip(da, db): votes[x][y] += 1
    dm = {}; used = collections.Counter()
    for x, c in votes.items():
        (y, v), *rest = c.most_common(2)
        if rest and rest[0][1] * 2 > v: continue
        dm[x] = (y, v)
    inv = collections.defaultdict(list)
    for x, (y, v) in dm.items(): inv[y].append((v, x))
    data = {}
    for y, xs in inv.items():
        xs.sort(reverse=True)
        if len(xs) == 1 or xs[0][0] > 2 * xs[1][0]: data[xs[0][1]] = y
    print(f'round {rnd}: data pairs {len(data)}', flush=True)

# read GM exe for vtable slots: use A function set; GM image from feat names is not available, so read the GM exe beside the PDB
gm_exe = os.environ.get('GM_EXE')
gd = open(gm_exe, 'rb').read()
gpe = struct.unpack_from('<I', gd, 0x3c)[0]; gn = struct.unpack_from('<H', gd, gpe+6)[0]; gopt = struct.unpack_from('<H', gd, gpe+20)[0]
gsec = [struct.unpack_from('<8sIIII', gd, gpe+24+gopt+40*i) for i in range(gn)]
def grd(v):
    r = v - 0x400000
    for nm, vs, sva, rs, rp in gsec:
        if sva <= r < sva + rs: return struct.unpack_from('<I', gd, rp + r - sva)[0]
    return None
nv = nf = 0
for x, y in data.items():
    if not dpub.get(x, '').startswith('??_7'): continue
    nv += 1
    for i in range(4000):
        fa_, fb_ = grd(x + 4*i), rd(y + 4*i)
        if fa_ not in A or fb_ not in B: break
        if i and (x + 4*i in dpub or x + 4*i in gdata): break
        if fa_ not in m and fb_ not in rm:
            sa, sb = A[fa_]['size'], B[fb_]['size']
            if min(sa, sb) / max(sa, sb, 1) >= 0.5:
                m[fa_] = fb_; rm[fb_] = fa_; how[fa_] = 'vtslot'; nf += 1
print('vtables matched', nv, 'new functions from vtable slots', nf)
named = sum(1 for x in data if x in gdata or x in dpub)
print('data pairs', len(data), 'with a source symbol', named)
json.dump([dict(a=a, b=b, name=A[a]['name'], how=how[a], size=B[b]['size']) for a, b in m.items()], open(mout, 'w'))
json.dump([dict(a=x, b=y) for x, y in data.items() if x in gdata or x in dpub], open(dout, 'w'))
