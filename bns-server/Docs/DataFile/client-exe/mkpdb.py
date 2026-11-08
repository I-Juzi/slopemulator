# Build a PDB for a target exe from a source PDB (types + symbols) and a function match list.
# usage: python3 -I mkpdb.py SRC_4K.pdb TARGET.exe MATCH.json OUT.pdb [--src-exe SRC.exe]
#   MATCH.json: [{"a": src_va, "b": dst_va, "how": "..."}...]  ("how"=="bytes" -> copy locals)
# The output keeps the source TPI stream (types) and writes new DBI/publics/globals/symrec
# and one module stream with S_GPROC32 records at the target addresses.
import sys, os, struct, json, collections
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import pdbio
from pdbio import sym

src, tgt, mpath, outp = sys.argv[1:5]
st = pdbio.msf_read(src)
dbi = pdbio.parse_dbi(st[3])

def pe_info(path):
    d = open(path, 'rb').read()
    pe = struct.unpack_from('<I', d, 0x3c)[0]
    nsec = struct.unpack_from('<H', d, pe + 6)[0]; opt = struct.unpack_from('<H', d, pe + 20)[0]
    base = struct.unpack_from('<I', d, pe + 24 + 28)[0]; ts = struct.unpack_from('<I', d, pe + 8)[0]
    sh = d[pe + 24 + opt: pe + 24 + opt + 40 * nsec]
    rs = d.find(b'RSDS'); guid = age = None
    while rs != -1:
        if b'Client' in d[rs+24:rs+300].split(b'\0')[0] or True:
            guid = d[rs+4:rs+20]; age = struct.unpack_from('<I', d, rs+20)[0]
        rs = d.find(b'RSDS', rs + 1)
    return base, sh, ts, guid, age

def secs(sh):
    out = []
    for i in range(len(sh) // 40):
        name = sh[i*40:i*40+8].rstrip(b'\0'); vs, va = struct.unpack_from('<II', sh, i*40+8)
        ch = struct.unpack_from('<I', sh, i*40+36)[0]
        out.append((name, va, vs, ch))
    return out

# source sections from the PDB section header stream
src_secs = secs(st[dbi['dbg'][5]])
src_base = 0x400000
tbase, tsh, tts, tguid, tage = pe_info(tgt)
tsecs = secs(tsh)
if '--guid-from-exe' not in sys.argv:
    pass
# the RSDS for Client.pdb is the last one in Client.exe (wiper.pdb comes first)

def va2so(va, base, ss):
    r = va - base
    for i, (_, sva, svs, _) in enumerate(ss):
        if sva <= r < sva + max(svs, 1): return i + 1, r - sva
    return None
def so2va(seg, off, base, ss):
    return base + ss[seg-1][1] + off

# ---- collect source symbols keyed by VA
procs = {}      # va -> (kind, typind, flags, name, subtree records)
for mi, m in enumerate(dbi['mods']):
    if m['stream'] == 0xffff: continue
    buf = st[m['stream']]
    recs = list(pdbio.iter_syms(buf, 4, m['symsz']))
    i = 0
    while i < len(recs):
        o, k, p = recs[i]
        if k in (0x1110, 0x110f):
            pParent, pEnd, pNext, ln, ds, de, ti, off, seg, fl = struct.unpack_from('<IIIIIIIIHB', p, 0)
            name = p[35:p.index(b'\0', 35)]
            # gather subtree until matching end
            sub = []; depth = 1; j = i + 1
            while j < len(recs) and depth:
                o2, k2, p2 = recs[j]
                if k2 in (0x1103, 0x1110, 0x110f, 0x1102, 0x114d): depth += 1
                elif k2 in (0x0006, 0x114e): depth -= 1
                if depth: sub.append((k2, p2, depth))
                j += 1
            va = so2va(seg, off, src_base, src_secs)
            procs[va] = (k, ti, ln, ds, de, fl, name, sub)
            i = j; continue
        i += 1
pubs = {}
gdatas = {}
glob_keep = []  # type-only global symbols (S_UDT, S_CONSTANT)
for o, k, p in pdbio.iter_syms(st[dbi['symrec']]):
    if k == 0x110e:
        fl, off, seg = struct.unpack_from('<IIH', p, 0)
        if 1 <= seg <= len(src_secs):
            pubs.setdefault(so2va(seg, off, src_base, src_secs), (fl, p[10:p.index(b'\0', 10)]))
    elif k in (0x1108, 0x1107):
        glob_keep.append((k, p))
    elif k in (0x110d, 0x110c):
        ti, off, seg = struct.unpack_from('<IIH', p, 0)
        if 1 <= seg <= len(src_secs): gdatas.setdefault(so2va(seg, off, src_base, src_secs), (k, ti, p[10:p.index(b'\0', 10)]))
print('source procs', len(procs), 'publics', len(pubs), 'udt/const', len(glob_keep), flush=True)

M = [dict(a=va, b=va, how='bytes') for va in set(procs) | set(pubs)] if mpath == 'IDENTITY' else json.load(open(mpath))
# ---- module stream
mod = bytearray(struct.pack('<I', 4))
mod += sym(0x1101, struct.pack('<I', 0) + b'Client_symbols.obj\0')
procrefs = []   # (name, offset in module)
pubs_out = []   # (seg, off, flags, name)
nproc = nlocal = 0
lo = None
for e in sorted(M, key=lambda e: e['b']):
    so = va2so(e['b'], tbase, tsecs)
    if so is None: continue
    seg, off = so
    pr = procs.get(e['a']); pb = pubs.get(e['a'])
    if pr:
        k, ti, ln, ds, de, fl, name, sub = pr
        if e.get('size'): ln = e['size']
        start = len(mod)
        body = bytearray()
        if e.get('how') == 'bytes':
            for k2, p2, depth in sub:
                if k2 in (0x1012, 0x110b, 0x1111, 0x1106, 0x113a, 0x1108, 0x1107, 0x113e, 0x1141, 0x1142, 0x1143, 0x1144, 0x1145):
                    body += sym(k2, p2); nlocal += 1
        rec = sym(k, struct.pack('<IIIIIIIIHB', 0, 0, 0, ln, ds, de, ti, off, seg, fl) + name + b'\0')
        endoff = start + len(rec) + len(body)
        rec = bytearray(rec); struct.pack_into('<I', rec, 8, endoff)
        mod += rec + body + sym(0x0006, b'')
        procrefs.append((name, start, k)); nproc += 1
    if pb:
        pubs_out.append((seg, off, pb[0], pb[1]))
    elif pr is None and e.get('name') and not e['name'].startswith(('sub_', 'nullsub', 'j_', 'unknown')):
        pubs_out.append((seg, off, 2, e['name'].encode()))
ndata = 0; data_out = []
if '--data' in sys.argv:
    for e in json.load(open(sys.argv[sys.argv.index('--data') + 1])):
        so = va2so(e['b'], tbase, tsecs)
        if so is None: continue
        if e['a'] in gdatas: data_out.append((so, gdatas[e['a']])); ndata += 1
        pb = pubs.get(e['a'])
        if pb and not pb[0] & 2: pubs_out.append((so[0], so[1], pb[0], pb[1]))
print('data symbols', ndata, flush=True)
modsyms = bytes(mod)
modstream = modsyms + struct.pack('<I', 0)   # no C11/C13, global refs size 0
print('procs written', nproc, 'locals', nlocal, 'publics', len(pubs_out), flush=True)

# ---- symrec + globals + publics
symrec = bytearray(); gents = []; pents = []
for name, off, k in procrefs:
    o = len(symrec)
    symrec += sym(0x1125 if k == 0x1110 else 0x1127, struct.pack('<IIH', 0, off, 1) + name + b'\0')
    gents.append((o, name))
for (seg, off), (k, ti, name) in data_out:
    o = len(symrec); symrec += sym(k, struct.pack('<IIH', ti, off, seg) + name + b'\0')
    gents.append((o, name))
for k, p in glob_keep:
    o = len(symrec); symrec += sym(k, p)
    nm = p[4:] if k == 0x1108 else None
    if k == 0x1108: nm = p[4:p.index(b'\0', 4)]
    else:
        # S_CONSTANT: typind u32, numeric leaf, name
        v = struct.unpack_from('<H', p, 4)[0]
        q = 6 if v < 0x8000 else 6 + {0x8000:1,0x8001:2,0x8002:2,0x8003:4,0x8004:4,0x8009:8,0x800a:8}.get(v, 0)
        nm = p[q:p.index(b'\0', q)]
    gents.append((o, nm))
addrmap = []
for seg, off, fl, name in pubs_out:
    o = len(symrec); symrec += sym(0x110e, struct.pack('<IIH', fl, off, seg) + name + b'\0')
    pents.append((o, name)); addrmap.append((seg, off, o))
addrmap.sort()
psi_hash = pdbio.gsi_hash(pents)
am = struct.pack(f'<{len(addrmap)}I', *[o for _, _, o in addrmap])
psi = struct.pack('<IIIIHHII', len(psi_hash), len(am), 0, 0, 0, 0, 0, 0) + psi_hash + am
gsi = pdbio.gsi_hash(gents)

# ---- stream layout: keep source indices for PDB info, TPI, TPI hash, /names, headerblock
keep = {1, 2, 4}
tpihash = struct.unpack_from('<H', st[2], 20)[0]; keep.add(tpihash)
p1 = st[1]
o = 28; strsz = struct.unpack_from('<I', p1, o)[0]
keep |= {277, 10} if len(st) > 277 else set()
streams = [b'' if (i not in keep or s is None) else s for i, s in enumerate(st)]
streams[0] = b''
n0 = len(streams)
I_GSI, I_PSI, I_SYMREC, I_SECHDR, I_MOD = n0, n0 + 1, n0 + 2, n0 + 3, n0 + 4
streams += [gsi, psi, bytes(symrec), tsh, modstream]

# ---- PDB info stream: target GUID/age/signature
guid, age = tguid, tage
if '--src-guid' in sys.argv:
    guid = p1[12:28]; age = struct.unpack_from('<I', p1, 8)[0]
p1 = bytearray(p1); struct.pack_into('<II', p1, 4, tts, age); p1[12:28] = guid
streams[1] = bytes(p1)

# ---- DBI
mi = bytearray(struct.pack('<I', 0))
first = sorted(M, key=lambda e: e['b'])
sc = struct.pack('<HHiiIHHII', 0xffff, 0, 0, 0, 0, 0, 0, 0, 0)
mi += sc + struct.pack('<HHIIIHHIII', 0, I_MOD, len(modsyms), 0, 0, 0, 0, 0, 0, 0)
mi += b'Client_symbols.obj\0' + b'Client_symbols.obj\0'
mi += b'\0' * ((-len(mi)) % 4)
modinfo = bytes(mi)
seccontr = struct.pack('<I', 0xeffe0000 + 19970605)
# section map
smap = bytearray(struct.pack('<HH', len(tsecs) + 1, len(tsecs) + 1))
for i, (name, va, vs, ch) in enumerate(tsecs):
    fl = 0x108  # selector | 32-bit
    if ch & 0x40000000: fl |= 1
    if ch & 0x80000000: fl |= 2
    if ch & 0x20000000: fl |= 4
    smap += struct.pack('<HHHHHHII', fl, 0, 0, i + 1, 0xffff, 0xffff, 0, vs)
smap += struct.pack('<HHHHHHII', 0x208, 0, 0, len(tsecs) + 1, 0xffff, 0xffff, 0, 0xffffffff)
fileinfo = struct.pack('<HHHH', 1, 0, 0, 0)
ecnames = struct.pack('<III', 0xEFFEEFFE, 1, 4) + b'\0\0\0\0' + struct.pack('<III', 1, 0, 0)
dbg = [0xffff] * 11; dbg[5] = I_SECHDR
dbgb = struct.pack('<11H', *dbg)
h = struct.pack('<iIIHHHHHHiiiiiIiiHHI', -1, 19990903, age, I_GSI, 0x8e00, I_PSI, 0, I_SYMREC, 0,
                len(modinfo), len(seccontr), len(smap), len(fileinfo), 0, 0, len(dbgb), len(ecnames), 0, 0x14c, 0)
streams[3] = h + modinfo + seccontr + bytes(smap) + fileinfo + ecnames + dbgb

nb = pdbio.msf_write(outp, streams)
print('wrote', outp, nb * 4096, 'bytes', 'guid', guid.hex(), 'age', age, flush=True)
