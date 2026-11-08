import struct, zlib, io, contextlib, sys, collections
sys.path.insert(0,'.')
with contextlib.redirect_stdout(io.StringIO()):
    from bodycheck2 import tables
f, w = sys.argv[1], int(sys.argv[2]); NO = 20 if w == 32 else 24; SO = 16
C = collections.Counter()
def nul_end(buf, s, e):
    k = s
    while k + 2 <= e and buf[k:k+2] != b'\0\0': k += 2
    return k
for kind, typ, body in tables(f, w, False):
    if typ not in (327, 233): continue
    body = bytes(body)
    def recs():
        if body[0] == 0:
            hs = 14; fx, ex = struct.unpack_from('<II', body, hs-9); c = hs
            while c < hs+fx:
                sz = struct.unpack_from('<H', body, c+4)[0]; yield body, c, hs+fx, hs+fx+ex, 'plain'; c += sz
        else:
            nb = struct.unpack_from('<I', body, 1)[0]; c = 7
            for _ in range(nb):
                c += 16; cs = struct.unpack_from('<H', body, c)[0]; c += 2; blk = zlib.decompress(body[c:c+cs]); c += cs
                ds, rc = struct.unpack_from('<HI', body, c); c += 6; offs = struct.unpack_from('<%dH'%rc, body, c); c += 2*rc
                for i, o in enumerate(offs):
                    end = offs[i+1] if i+1 < rc else ds; sz = struct.unpack_from('<H', blk, o+4)[0]
                    yield blk, o, o+sz, end, 'comp'
    for buf, o, lo, hi, mode in recs():
        a, b = struct.unpack_from('<II', buf, o+NO)
        if w == 64: C[(typ, mode, 'x64 third word zero', struct.unpack_from('<I', buf, o+NO+8)[0] == 0)] += 1
        s = lo + b
        ok_start = b % 2 == 0 and s <= hi and (b == 0 or buf[s-2:s] == b'\0\0')
        e = nul_end(buf, s, hi)
        C[(typ, mode, 'text start ok', ok_start)] += 1
        if mode == 'comp': C[(typ, mode, 'text ends at span end', e + 2 == hi)] += 1
        n = (e - s) // 2
        C[(typ, mode, 'a vs text chars', 'a==chars' if a == n else 'a==chars+1' if a == n+1 else 'a==bytes' if a == 2*n else 'a==bytes+2' if a == 2*n+2 else 'a<chars' if a < n else 'a>chars')] += 1
for k, v in sorted(C.items(), key=str): print(k, v)
