import struct, sys, json, glob, collections
sys.path.insert(0, '.')
exec(open('findtd.py').read().split("for name in")[0])
J = {}
for p in glob.glob('/home/ubuntu/Downloads/TableDefs_0.0.381.8876_x86 KR TEST/*.json'):
    x = json.load(open(p)); J[x['type']] = x
U = lambda f, o: struct.unpack_from(f, d, o)
def wstr(v):
    o = va2off(v); e = o
    while d[e:e+2] != b'\0\0': e += 2
    return d[o:e].decode('utf-16-le')
ARR = va2off(0x194ece8)
S = collections.Counter(); tmap = collections.defaultdict(set); diffs = []
def chk(what, a, b):
    if a == b: S['ok'] += 1
    else: S['diff ' + what] += 1; diffs.append((what, a, b))
def attrs(av, n, ja):
    chk('attrcount', n, len(ja or []))
    for k in range(min(n, len(ja or []))):
        o = va2off(av) + 64 * k; a = ja[k]
        chk('attr name', wstr(U('<I', o)[0]), a['name'])
        t, key, rep, req = U('<BBBB', o + 8); tmap[a['type']].add(t)
        chk('key', bool(key), bool(a['key'])); chk('repeat', rep, a['repeat']); chk('required', req, a['required'])
        sc, rt, re_, dep, hid = U('<HhBBB', o + 44)
        chk('seqcount', sc, a.get('seqcount', 0)); chk('reftable', rt, a['reftable']); chk('refel', re_, a['refel'])
        chk('deprecated', bool(dep), a['deprecated']); chk('hidden', bool(hid), a['hidden'])
        off, sz = U('<HH', o + 52); chk('offset', off, a['offset']); chk('size', sz, a['size'])
for tid in range(363):
    tv = U('<I', ARR + 4 * tid)[0]
    if tid == 0: chk('slot0', tv, 0); continue
    j = J[tid]; o = va2off(tv)
    nm, typ, maj, mnr, elv, elc = U('<IhhhxxIB', o)
    chk('name', wstr(nm), j['name']); chk('type', typ, tid); chk('ver', (maj, mnr), (j['major_ver'], j['minor_ver'])); chk('elcount', elc, j['elcount'])
    for e in range(elc):
        eo = va2off(elv) + 44 * e; je = j['el'][e]
        en, et, bv = U('<IBxxxI', eo); child = list(d[eo+12:eo+44])
        chk('el name', wstr(en), je['name']); chk('el type', et, je['type']); chk('child', child, je['child'])
        bo = va2off(bv); jb = je['body']
        av, sv, dv, ac, scn, size, ko = U('<IIIhhHH', bo)
        chk('body size', size, jb['size']); chk('keyoffset', ko, jb['keyoffset']); chk('subcount', scn, jb['subcount'])
        attrs(av, ac, jb['attr'])
        for s in range(scn):
            so = va2off(sv) + 16 * s; js = jb['sub'][s]
            sn, sav, sdv, sac, ssz = U('<IIIhH', so)
            chk('sub name', wstr(sn), js['name']); chk('sub size', ssz, js['size'])
            attrs(sav, sac, js['attr'])
print(dict(S)); print(diffs[:10])
print('type codes:', {k: sorted(v) for k, v in sorted(tmap.items(), key=lambda x: min(x[1]))})
