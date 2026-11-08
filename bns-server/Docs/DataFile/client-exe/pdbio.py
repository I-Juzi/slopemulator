# Minimal PDB (MSF 7.00) reader/writer and symbol helpers.
import struct

MAGIC = b'Microsoft C/C++ MSF 7.00\r\n\x1aDS\0\0\0'

def msf_read(path):
    d = open(path, 'rb').read()
    assert d[:32] == MAGIC
    bs, fpm, nblocks, dirbytes, _, bmap = struct.unpack_from('<6I', d, 32)
    ndir = -(-dirbytes // bs)
    dl = struct.unpack_from(f'<{ndir}I', d, bmap * bs)
    dirb = b''.join(d[b*bs:(b+1)*bs] for b in dl)[:dirbytes]
    ns = struct.unpack_from('<I', dirb, 0)[0]
    sizes = struct.unpack_from(f'<{ns}I', dirb, 4)
    off = 4 + 4*ns; streams = []
    for s in sizes:
        if s == 0xFFFFFFFF: streams.append(None); continue
        n = -(-s // bs)
        bl = struct.unpack_from(f'<{n}I', dirb, off); off += 4*n
        streams.append(b''.join(d[b*bs:(b+1)*bs] for b in bl)[:s])
    return streams

def msf_write(path, streams, bs=4096):
    out = bytearray(bs * 3)
    def alloc(n):
        res = []
        while len(res) < n:
            b = len(out) // bs
            out.extend(b'\0' * bs)
            if b % bs in (1, 2):  # free page map blocks
                out[b*bs:(b+1)*bs] = b'\xff' * bs if False else b'\0' * bs
                continue
            res.append(b)
        return res
    def write(blob):
        n = -(-len(blob) // bs) if blob else 0
        bl = alloc(n)
        for i, b in enumerate(bl):
            c = blob[i*bs:(i+1)*bs]; out[b*bs:b*bs+len(c)] = c
        return bl
    d = struct.pack('<I', len(streams)) + b''.join(struct.pack('<I', 0xFFFFFFFF if s is None else len(s)) for s in streams)
    for s in streams:
        if s: d += struct.pack(f'<{-(-len(s)//bs)}I', *write(s))
    dbl = write(d)
    assert len(dbl) * 4 <= bs
    bm = write(struct.pack(f'<{len(dbl)}I', *dbl))[0]
    total = len(out) // bs
    # FPM: bit set = free. Everything allocated is used; mark nothing free.
    out[0:56] = MAGIC + struct.pack('<6I', bs, 1, total, len(d), 0, bm)
    open(path, 'wb').write(out)
    return total

def cstr(b, o):
    e = b.index(b'\0', o); return b[o:e], e + 1

def iter_syms(buf, start=0, end=None):
    o = start; end = len(buf) if end is None else end
    while o + 4 <= end:
        ln, kind = struct.unpack_from('<HH', buf, o)
        if ln < 2: break
        yield o, kind, buf[o+4:o+2+ln]
        o += 2 + ln

def parse_dbi(dbi):
    (vsig, vhdr, age, gsi, build, psi, dllver, symrec, rbld, modsz, sczs, smsz, sisz, tsmsz,
     mfc, dbgsz, ecsz, flags, machine, pad) = struct.unpack_from('<iIIHHHHHHiiiiiIiiHHI', dbi, 0)
    o = 64; mods = []
    end = o + modsz
    while o < end:
        sc = dbi[o+4:o+32]
        mflags, mstream, symsz, c11, c13, nfiles = struct.unpack_from('<HHIIIH', dbi, o+32)
        name, p = cstr(dbi, o + 64); obj, p = cstr(dbi, p)
        mods.append(dict(stream=mstream, symsz=symsz, c11=c11, c13=c13, name=name, obj=obj))
        o = (p + 3) & ~3
    o = 64 + modsz + sczs + smsz + sisz + tsmsz + ecsz
    dbg = struct.unpack_from(f'<{dbgsz//2}H', dbi, o)
    return dict(age=age, gsi=gsi, psi=psi, symrec=symrec, mods=mods, dbg=dbg, machine=machine, build=build, dllver=dllver)

def hash_v1(s: bytes):
    r = 0; n = len(s)
    for i in range(0, n - n % 4, 4): r ^= struct.unpack_from('<I', s, i)[0]
    i = n - n % 4; rem = n % 4
    if rem >= 2: r ^= struct.unpack_from('<H', s, i)[0]; i += 2; rem -= 2
    if rem == 1: r ^= s[i]
    r |= 0x20202020; r ^= r >> 11
    return (r ^ (r >> 16)) & 0xFFFFFFFF

def _gsi_less_key(name: bytes):
    try:
        name.decode('ascii'); return (len(name), 0, name.lower())
    except UnicodeDecodeError:
        return (len(name), 1, name)

def gsi_hash(entries):
    """entries: list of (symrec_offset, name bytes). Returns GSI hash table bytes."""
    IPHR = 4096
    buckets = [[] for _ in range(IPHR + 1)]
    for off, name in entries:
        buckets[hash_v1(name) % IPHR].append((off, name))
    recs = []; starts = []; bitmap = [0] * ((IPHR + 32) // 32)
    for i, b in enumerate(buckets):
        if not b: continue
        b.sort(key=lambda e: _gsi_less_key(e[1]))
        bitmap[i // 32] |= 1 << (i % 32)
        starts.append(len(recs) * 12)
        recs.extend(b)
    hr = b''.join(struct.pack('<II', off + 1, 1) for off, _ in recs)
    bk = struct.pack(f'<{len(bitmap)}I', *bitmap) + struct.pack(f'<{len(starts)}I', *starts)
    return struct.pack('<IIII', 0xFFFFFFFF, 0xF12F091A, len(hr), len(bk)) + hr + bk

def sym(kind, payload):
    rec = struct.pack('<H', kind) + payload
    rec += b'\0' * ((-(len(rec) + 2)) % 4)
    return struct.pack('<H', len(rec)) + rec
