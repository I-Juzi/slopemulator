"""Export table definitions from a client executable into TableDefs-style JSON.

Passive static read of the PE file. Usage:
  python3 -I exportdefs.py EXE ARRAY_VA LAYOUT OUTDIR
LAYOUT: x64p4 (2026 BNSR_116, 4-byte packed) or x86 (0.0.381 Client.exe).
"""
import json, os, struct, sys

TYPES = {1: 'int8', 2: 'int16', 3: 'int32', 4: 'int64', 5: 'float32', 6: 'bool', 7: 'string',
         8: 'seq', 9: 'seq16', 10: 'ref', 11: 'tref', 12: 'sub', 15: 'vector32', 16: 'icolor',
         18: 'box', 20: 'msec', 21: 'distance', 22: 'velocity', 23: 'prop_seq', 24: 'prop_field',
         25: 'script_obj', 26: 'native', 28: 'icon', 30: 'time64'}

# Offsets per structure. P = pointer size.
LAYOUTS = {
    'x64p4': dict(P=8,
        td=dict(name=0, type=8, major=10, minor=12, el=16, elcount=24, maxid=28, autokey=36, module=40),
        el=dict(stride=52, name=0, type=8, body=12, child=20),
        body=dict(stride=32, attr=0, sub=8, defrec=16, attrcount=24, subcount=26, size=28, keyoffset=30),
        sub=dict(stride=28, name=0, attr=8, defrec=16, attrcount=24, size=26),
        attr=dict(stride=84, name=0, namehash=8, type=16, key=17, repeat=18, required=19, defvalue=20,
                  min=28, max=36, fmin=44, fmax=48, seq=52, seqcount=60, reftable=62, refel=64,
                  deprecated=65, hidden=66, offset=68, size=70, propfield=72),
        nh=dict(hash=0, name=4)),
    'x86': dict(P=4,
        td=dict(name=0, type=4, major=6, minor=8, el=12, elcount=16, maxid=20, autokey=28, module=32),
        el=dict(stride=44, name=0, type=4, body=8, child=12),
        body=dict(stride=20, attr=0, sub=4, defrec=8, attrcount=12, subcount=14, size=16, keyoffset=18),
        sub=dict(stride=16, name=0, attr=4, defrec=8, attrcount=12, size=14),
        attr=dict(stride=64, name=0, namehash=4, type=8, key=9, repeat=10, required=11, defvalue=12,
                  min=16, max=24, fmin=32, fmax=36, seq=40, seqcount=44, reftable=46, refel=48,
                  deprecated=49, hidden=50, offset=52, size=54, propfield=56),
        nh=dict(hash=0, name=4)),
}


class Image:
    def __init__(self, path):
        self.d = open(path, 'rb').read()
        d = self.d
        pe = struct.unpack_from('<I', d, 0x3c)[0]
        nsec, = struct.unpack_from('<H', d, pe + 6)
        optsz, = struct.unpack_from('<H', d, pe + 20)
        magic, = struct.unpack_from('<H', d, pe + 24)
        self.base = struct.unpack_from('<Q' if magic == 0x20b else '<I', d, pe + 24 + (24 if magic == 0x20b else 28))[0]
        self.secs = []
        for i in range(nsec):
            o = pe + 24 + optsz + 40 * i
            vs, va, rs, ra = struct.unpack_from('<IIII', d, o + 8)
            self.secs.append((va, vs, ra, rs))

    def off(self, v):
        if not v:
            return None
        r = v - self.base
        for va, vs, ra, rs in self.secs:
            if va <= r < va + rs:
                return ra + r - va
        return None

    def u(self, fmt, v, k=0):
        o = self.off(v)
        if o is None:
            raise ValueError('bad address %#x' % v)
        return struct.unpack_from('<' + fmt, self.d, o + k)[0]

    def wstr(self, v):
        o = self.off(v)
        if o is None:
            return None
        e = o
        while self.d[e:e + 2] != b'\0\0':
            e += 2
            if e - o > 4096:
                raise ValueError('unterminated string at %#x' % v)
        return self.d[o:e].decode('utf-16-le')


def export(img, array_va, L):
    P = 'Q' if L['P'] == 8 else 'I'
    td, el, bd, sb, at, nh = (L[k] for k in ('td', 'el', 'body', 'sub', 'attr', 'nh'))
    unknown_types = set()

    def defrec(v):
        if not img.off(v):
            return None
        return {'type': img.u('b', v), 'subtype': img.u('h', v, 2), 'size': img.u('H', v, 4)}

    def attrs(v, n):
        out = []
        for i in range(n):
            a = v + at['stride'] * i
            t = img.u('B', a, at['type'])
            if t not in TYPES:
                unknown_types.add(t)
            hv = img.u(P, a, at['namehash'])
            seqv, seqn = img.u(P, a, at['seq']), img.u('H', a, at['seqcount'])
            dv = img.u(P, a, at['defvalue'])
            out.append({
                'name': img.wstr(img.u(P, a, at['name'])),
                'namehash': None if not img.off(hv) else {
                    'namehash': img.u('I', hv, nh['hash']),
                    'name': img.wstr(img.u(P, hv, nh['name']))},
                'type': TYPES.get(t, 'type_%d' % t),
                'key': bool(img.u('B', a, at['key'])),
                'repeat': img.u('B', a, at['repeat']),
                'required': img.u('B', a, at['required']),
                'defvalue': img.wstr(dv) if img.off(dv) else None,
                'min': img.u('q', a, at['min']), 'max': img.u('q', a, at['max']),
                'fmin': img.u('f', a, at['fmin']), 'fmax': img.u('f', a, at['fmax']),
                'seq': [img.wstr(img.u(P, seqv + L['P'] * k)) for k in range(seqn)] if img.off(seqv) else None,
                'seqcount': seqn,
                'reftable': img.u('h', a, at['reftable']), 'refel': img.u('B', a, at['refel']),
                'deprecated': bool(img.u('B', a, at['deprecated'])),
                'hidden': bool(img.u('B', a, at['hidden'])),
                'offset': img.u('H', a, at['offset']), 'size': img.u('H', a, at['size']),
                'propfield': img.u('I', a, at['propfield']),
            })
        return out or None

    tables = []
    i = 1
    while True:
        tv = img.u(P, array_va + L['P'] * i) if img.off(array_va + L['P'] * i) is not None else 0
        try:
            ok = bool(img.off(tv)) and img.u('h', tv, td['type']) == i and bool(img.wstr(img.u(P, tv, td['name'])))
        except (ValueError, UnicodeDecodeError, struct.error):
            ok = False
        if not ok:
            break
        elv, elc = img.u(P, tv, td['el']), img.u('B', tv, td['elcount'])
        els = []
        for e in range(elc):
            ev = elv + el['stride'] * e
            bv = img.u(P, ev, el['body'])
            ac, sc = img.u('h', bv, bd['attrcount']), img.u('h', bv, bd['subcount'])
            subs = []
            for s in range(sc):
                sv = img.u(P, bv, bd['sub']) + sb['stride'] * s
                sac = img.u('H', sv, sb['attrcount'])
                subs.append({'name': img.wstr(img.u(P, sv, sb['name'])),
                             'attr': attrs(img.u(P, sv, sb['attr']), sac),
                             'defaultRecord': defrec(img.u(P, sv, sb['defrec'])),
                             'attrcount': sac, 'size': img.u('H', sv, sb['size'])})
            els.append({'name': img.wstr(img.u(P, ev, el['name'])),
                        'type': img.u('I' if L['P'] == 8 else 'B', ev, el['type']),
                        'body': {'attr': attrs(img.u(P, bv, bd['attr']), ac), 'sub': subs or None,
                                 'defaultRecord': defrec(img.u(P, bv, bd['defrec'])),
                                 'attrcount': ac, 'subcount': sc,
                                 'size': img.u('H', bv, bd['size']),
                                 'keyoffset': img.u('H', bv, bd['keyoffset'])},
                        'child': [img.u('B', ev, el['child'] + k) for k in range(32)]})
        tables.append({'name': img.wstr(img.u(P, tv, td['name'])), 'type': img.u('h', tv, td['type']),
                       'major_ver': img.u('h', tv, td['major']), 'minor_ver': img.u('h', tv, td['minor']),
                       'el': els, 'elcount': elc, 'maxid': img.u('I', tv, td['maxid']),
                       'autokey': bool(img.u('B', tv, td['autokey'])),
                       'module': img.u('I', tv, td['module'])})
        i += 1
    return tables, unknown_types


if __name__ == '__main__':
    exe, arr, layout, outdir = sys.argv[1], int(sys.argv[2], 0), sys.argv[3], sys.argv[4]
    img = Image(exe)
    tables, unk = export(img, arr, LAYOUTS[layout])
    os.makedirs(outdir, exist_ok=True)
    for t in tables:
        with open(os.path.join(outdir, t['name'] + '.json'), 'w') as f:
            json.dump(t, f, indent=2, ensure_ascii=False)
    print('tables', len(tables), 'unknown type codes', sorted(unk))
