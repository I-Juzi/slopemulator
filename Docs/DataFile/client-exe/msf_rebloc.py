# Rewrite an MSF 7.00 PDB with a larger block size (stream contents unchanged).
import sys, struct
src, dst, nbs = sys.argv[1], sys.argv[2], int(sys.argv[3]) if len(sys.argv) > 3 else 4096
d = open(src, 'rb').read()
MAGIC = b'Microsoft C/C++ MSF 7.00\r\n\x1aDS\0\0\0'
assert d[:32] == MAGIC
bs, fpm, nblocks, dirbytes, _, bmap = struct.unpack_from('<6I', d, 32)
ndir = -(-dirbytes // bs)
# directory block list: contiguous u32 array starting at block bmap
dl = struct.unpack_from(f'<{ndir}I', d, bmap * bs)
dirb = b''.join(d[b*bs:(b+1)*bs] for b in dl)[:dirbytes]
ns = struct.unpack_from('<I', dirb, 0)[0]
sizes = struct.unpack_from(f'<{ns}I', dirb, 4)
off = 4 + 4*ns
streams = []
for s in sizes:
    if s == 0xFFFFFFFF: streams.append(None); continue
    n = -(-s // bs)
    bl = struct.unpack_from(f'<{n}I', dirb, off); off += 4*n
    streams.append(b''.join(d[b*bs:(b+1)*bs] for b in bl)[:s])
assert off == dirbytes, (off, dirbytes)
print('streams', ns, 'old block', bs, 'total', sum(len(s) for s in streams if s))
# layout: 0 super, 1-2 FPM, then data
out = bytearray(nbs * 3)
def put(blob):
    global out
    first = len(out) // nbs
    out += blob + b'\0' * (-len(blob) % nbs)
    # keep FPM positions free: blocks k*nbs+1, k*nbs+2 are FPM
    return list(range(first, len(out)//nbs))
# simple: place blocks avoiding FPM intervals
blocks_used = []
def alloc(n):
    res = []
    while len(res) < n:
        b = len(out)//nbs
        out.extend(b'\0'*nbs)
        if b % nbs in (1, 2): continue
        res.append(b)
    return res
def write(blob):
    n = -(-len(blob)//nbs) if blob else 0
    bl = alloc(n)
    for i, b in enumerate(bl):
        chunk = blob[i*nbs:(i+1)*nbs]
        out[b*nbs:b*nbs+len(chunk)] = chunk
    return bl
newdir = struct.pack('<I', ns) + b''.join(struct.pack('<I', 0xFFFFFFFF if s is None else len(s)) for s in streams)
for s in streams:
    if s: newdir += struct.pack(f'<{-(-len(s)//nbs)}I', *write(s))
dbl = write(newdir)
assert len(dbl)*4 <= nbs, 'directory too large'
bm = write(struct.pack(f'<{len(dbl)}I', *dbl))[0]
total = len(out)//nbs
# free page map: mark all used (bit=0 used) -> zeros already; fpm blocks 1 and 2 at each interval
out[0:56] = MAGIC + struct.pack('<6I', nbs, 1, total, len(newdir), 0, bm)
open(dst, 'wb').write(out)
print('written', dst, total, 'blocks')
