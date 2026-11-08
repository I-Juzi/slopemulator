# Extract per-function features from an IDA database (read-only) to JSON lines.
# usage: python3 feat.py DB_INPUT_FILE OUT.jsonl
import sys, os, json, hashlib, time
sys.path.insert(0, os.path.expanduser('~/ida-pro-9.5/idalib/python'))
import idapro
path, out = sys.argv[1], sys.argv[2]
idapro.open_database(path, False)
import idautils, ida_funcs, ida_name, ida_bytes, ida_ua, ida_gdl, ida_segment, ida_nalt, idc, ida_xref

t0 = time.time()
segs = [(s.start_ea, s.end_ea) for s in (ida_segment.getnseg(i) for i in range(ida_segment.get_segm_qty()))]
lo, hi = min(a for a, _ in segs), max(b for _, b in segs)
def is_addr(v): return lo <= v < hi

def strlit(ea):
    t = ida_nalt.get_str_type(ea)
    if ida_bytes.is_strlit(ida_bytes.get_flags(ea)):
        s = ida_bytes.get_strlit_contents(ea, -1, t if t not in (-1, 0xffffffff, 0xffffffffffffffff) else 0)
        if s: return s.decode('utf-8', 'replace')
    return None

n = 0
with open(out, 'w') as fo:
    for fea in idautils.Functions():
        f = ida_funcs.get_func(fea)
        if f is None or f.flags & ida_funcs.FUNC_THUNK and False: continue
        mask = hashlib.md5(); mnem = hashlib.md5()
        strs, consts, calls, datarefs = [], [], [], []
        ninsn = 0
        for ea in idautils.FuncItems(fea):
            insn = ida_ua.insn_t()
            l = ida_ua.decode_insn(insn, ea)
            if l <= 0: continue
            ninsn += 1
            b = bytearray(ida_bytes.get_bytes(ea, l))
            # mask operands that hold addresses (relocations differ between builds)
            for op in insn.ops:
                if op.type == ida_ua.o_void: break
                if op.type in (ida_ua.o_mem, ida_ua.o_near, ida_ua.o_far, ida_ua.o_imm, ida_ua.o_displ):
                    v = op.addr if op.type in (ida_ua.o_mem, ida_ua.o_near, ida_ua.o_far, ida_ua.o_displ) else op.value
                    if op.type in (ida_ua.o_near, ida_ua.o_far) or is_addr(v):
                        off = op.offb or op.offo
                        if off:
                            for k in range(off, min(l, off + 4)): b[k] = 0
                    elif op.type == ida_ua.o_imm and v > 0xff and v not in (0xffffffff,):
                        consts.append(v & 0xffffffff)
            mask.update(bytes(b)); mnem.update(insn.get_canon_mnem().encode())
            for x in idautils.XrefsFrom(ea, 0):
                if x.type in (ida_xref.fl_CN, ida_xref.fl_CF):
                    calls.append(x.to)
                elif x.iscode == 0:
                    s = strlit(x.to)
                    if s is not None: strs.append(s)
                    else:
                        nm = ida_name.get_name(x.to)
                        datarefs.append(x.to)
        try:
            fc = ida_gdl.FlowChart(f); nb = fc.size; ne = sum(len(list(bb.succs())) for bb in fc)
        except Exception:
            nb = ne = 0
        rec = dict(ea=fea, size=f.end_ea - fea, name=ida_name.get_name(fea), lib=bool(f.flags & ida_funcs.FUNC_LIB),
                   thunk=bool(f.flags & ida_funcs.FUNC_THUNK), ninsn=ninsn, nb=nb, ne=ne,
                   mask=mask.hexdigest()[:16], mnem=mnem.hexdigest()[:16], strs=strs, consts=sorted(set(consts))[:64],
                   calls=calls, drefs=datarefs[:64])
        fo.write(json.dumps(rec) + '\n')
        n += 1
        if n % 20000 == 0: print(n, round(time.time() - t0), 's', flush=True)
    # vtables (named by PDB or by IDA's RTTI analysis): slot -> function
    nv = 0
    for ea, nm in idautils.Names():
        if not nm.startswith('??_7'): continue
        slots = []
        p = ea
        while True:
            v = ida_bytes.get_dword(p)
            if p != ea and ida_name.get_name(p).startswith('??_'): break
            if ida_funcs.get_func(v) is None or ida_funcs.get_func(v).start_ea != v: break
            slots.append(v); p += 4
            if len(slots) > 2000: break
        if slots:
            fo.write(json.dumps(dict(vt=nm, ea=ea, slots=slots)) + '\n'); nv += 1
    print('vtables', nv, flush=True)
print('done', n, round(time.time() - t0), 's', flush=True)
idapro.close_database(False)
