import struct, sys, math, collections
def pe(path):
    d = open(path, 'rb').read()
    o = struct.unpack_from('<I', d, 0x3c)[0]; mach, nsec = struct.unpack_from('<HH', d, o + 4)
    ts = struct.unpack_from('<I', d, o + 8)[0]; opt = o + 24; magic = struct.unpack_from('<H', d, opt)[0]
    base = struct.unpack_from('<I', d, opt + 28)[0] if magic == 0x10b else struct.unpack_from('<Q', d, opt + 24)[0]
    osz = struct.unpack_from('<H', d, o + 20)[0]; s = opt + osz; secs = []
    for i in range(nsec):
        name = d[s:s+8].rstrip(b'\0').decode('latin1'); vs, va, rs, ra = struct.unpack_from('<IIII', d, s + 8); ch = struct.unpack_from('<I', d, s + 36)[0]
        blk = d[ra:ra+rs]; c = collections.Counter(blk[:4_000_000]); n = sum(c.values()) or 1
        ent = -sum(v/n*math.log2(v/n) for v in c.values())
        secs.append((name, va, vs, ra, rs, ch, ent)); s += 40
    return d, ('x64' if magic == 0x20b else 'x86'), base, ts, secs
if __name__ == '__main__':
    import time
    for p in sys.argv[1:]:
        d, w, base, ts, secs = pe(p)
        print('==', p, w, 'base', hex(base), 'link time', time.strftime('%Y-%m-%d', time.gmtime(ts)))
        for n, va, vs, ra, rs, ch, e in secs: print('   %-8s va=%08x vsize=%9d rawsize=%9d ent=%.2f' % (n, va, vs, rs, e))
