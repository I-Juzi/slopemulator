# Test hypotheses for string/native field encoding. Prints counts only, never content.
import struct, sys, zlib, collections, json, glob
sys.path.insert(0, '.')
import io, contextlib
with contextlib.redirect_stdout(io.StringIO()):
    from bodycheck2 import tables

path, w, defdir = sys.argv[1], int(sys.argv[2]), sys.argv[3]
P = 4 if w == 32 else 8
D = {}
for f in glob.glob(defdir + '/*.json'):
    j = json.load(open(f)); D[j['type']] = j


def layout(td, et, st):
    for e in td['el']:
        if e['type'] == et:
            b = e['body']; a = list(b['attr'] or [])
            if st >= 0 and b['sub'] and st < len(b['sub']):
                a += b['sub'][st]['attr'] or []
            return a
    return None


C = collections.Counter()


def strok(buf, o, end):
    """o points at a UTF-16 string start: zero-terminated before end, and preceded by a terminator or region start."""
    if o < 0 or o + 2 > end or o % 2:
        return False
    e = o
    while e + 2 <= end and buf[e:e + 2] != b'\0\0':
        e += 2
    if e + 2 > end:
        return False
    return o == ex_start or buf[o - 2:o] == b'\0\0'


def probe(rec, ex_buf, ex_lo, ex_hi, mode):
    global ex_start
    ex_start = ex_lo
    t, _, st, sz = struct.unpack_from('<bBhH', rec, 0)
    a = layout(td, t, st)
    if a is None:
        C['no layout'] += 1; return
    for x in a:
        if x['type'] not in ('string', 'native'):
            continue
        for r in range(x['repeat']):
            o = x['offset'] + r * x['size']
            raw = bytes(rec[o:o + x['size']])
            if x['type'] == 'string':
                v = int.from_bytes(raw[:P], 'little')
                k = mode + ' string'
                if v == 0:
                    C[k + ' zero'] += 1; continue
                if mode == 'comp-record':
                    C[k + (' v>=fixed' if v >= sz else ' v<fixed')] += 1
                for name, base in (('byteoff', 0), ('charidx', 1)):
                    tgt = ex_lo + (v * 2 if base else v)
                    C[k + ' ' + name + (' ok' if strok(ex_buf, tgt, ex_hi) else ' bad')] += 1
            else:
                # native: try (ptr, u32) and (u32, ptr)
                v1 = int.from_bytes(raw[:P], 'little'); n1 = int.from_bytes(raw[P:P + 4], 'little')
                k = mode + ' native'
                if v1 == 0 and n1 == 0:
                    C[k + ' zero'] += 1; continue
                C[k + ' off+len fits' if ex_lo + v1 + n1 * 2 <= ex_hi else k + ' off+len no'] += 1
                C[k + ' off str ok' if strok(ex_buf, ex_lo + v1, ex_hi) else k + ' off str bad'] += 1
                if strok(ex_buf, ex_lo + v1, ex_hi):
                    e = ex_lo + v1
                    while ex_buf[e:e + 2] != b'\0\0': e += 2
                    C[k + ' len==chars' if (e - ex_lo - v1) // 2 == n1 else (k + ' len==chars+1' if (e - ex_lo - v1) // 2 + 1 == n1 else k + ' len other')] += 1


for kind, typ, body in tables(path, w, 'local' not in path):
    td = D[typ]; body = bytes(body)
    if body[0] == 0:
        hs = 18 if (kind == 1 and w == 64) else 14
        fx, ex = struct.unpack_from('<II', body, hs - 9)
        c = hs
        while c < hs + fx:
            sz = struct.unpack_from('<H', body, c + 4)[0]
            probe(memoryview(body)[c:c + sz], body, hs + fx, hs + fx + ex, 'plain')
            c += sz
    else:
        nb = struct.unpack_from('<I', body, 1)[0]; c = 7
        for _ in range(nb):
            c += 16; cs = struct.unpack_from('<H', body, c)[0]; c += 2
            blk = zlib.decompress(body[c:c + cs]); c += cs
            ds, rc = struct.unpack_from('<HI', body, c); c += 6
            offs = list(struct.unpack_from('<%dH' % rc, body, c)); c += 2 * rc
            for i, o in enumerate(offs):
                end = offs[i + 1] if i + 1 < rc else ds
                sz = struct.unpack_from('<H', blk, o + 4)[0]
                rec = memoryview(blk)[o:end]
                # strings relative to the extra part after the fixed record
                probe(rec, bytes(rec), sz, end - o, 'comp-extra')
                probe(rec, bytes(rec), 0, end - o, 'comp-record')

for k, v in sorted(C.items()):
    print('%-40s %d' % (k, v))
