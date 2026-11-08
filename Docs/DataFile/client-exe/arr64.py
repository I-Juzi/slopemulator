import struct, sys, collections
sys.path.insert(0, '.')
from find64 import *
U = lambda f, o: struct.unpack_from(f, d, o)
def wstr(v):
    o = va2off(v)
    if o is None: return None
    e = o
    while d[e:e+2] != b'\0\0' and e - o < 400: e += 2
    try: return d[o:e].decode('utf-16-le')
    except: return None
for r in refs(0x14667aa58):
    print('ptr at', hex(off2va(r)), sec(r))
    s = r
    while U('<Q', s - 8)[0] == 0 or (va2off(U('<Q', s - 8)[0]) is not None and wstr(U('<Q', va2off(U('<Q', s - 8)[0]))[0])): s -= 8
    n = 0; e = r
    while va2off(U('<Q', e)[0]) is not None and wstr(U('<Q', va2off(U('<Q', e)[0]))[0]): e += 8
    print('  array', hex(off2va(s)), 'slots', (e - s) // 8)
    ARR, N = s, (e - s) // 8
ok = 0; C = collections.Counter(); vers = []
for i in range(N):
    v = U('<Q', ARR + 8 * i)[0]
    if v == 0: C['null'] += 1; continue
    o = va2off(v); nm, typ, maj, mnr, el, elc = U('<QhhhxxQB', o)
    C['id==slot'] += typ == i; vers.append((typ, wstr(nm), maj, mnr, elc))
print(dict(C)); print(vers[:5], vers[-3:])
