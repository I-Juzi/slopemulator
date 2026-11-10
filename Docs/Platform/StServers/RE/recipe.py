# Service recipe for StServers binaries (see 00-framework.md, "Service recipe").
# Use in IDA: exec(fn_fingerprint.py); apply_names(framework-funcs.tsv); apply_names(framework-names.tsv);
# then exec(open(WT + "/Docs/Platform/StServers/RE/recipe.py").read()).
import re, idc, idautils, ida_bytes, ida_funcs, ida_hexrays, ida_nalt, ida_xref


def _arg(ea, reg, n=10):
    for _ in range(n):
        ea = idc.prev_head(ea)
        if idc.print_insn_mnem(ea) == "call": return None
        if idc.print_insn_mnem(ea) in ("lea", "mov") and idc.print_operand(ea, 0) == reg:
            return idc.get_operand_value(ea, 1)


def _wstr(ea):   # wide or narrow string literal at ea
    if not ea or ea == idc.BADADDR: return None
    wide = ida_bytes.get_byte(ea) and not ida_bytes.get_byte(ea + 1)
    s = ida_bytes.get_strlit_contents(ea, -1, ida_nalt.STRTYPE_C_16 if wide else ida_nalt.STRTYPE_C)
    return s.decode("utf-8", "replace") if s else None


# 1a. Name maps and the handlers found by the old switch scan (HelloSrv pattern).
def commands():
    init = idc.get_name_ea_simple("NameMapInit"); out = []
    for x in idautils.CodeRefsTo(init, 0):
        n, tb, hm = _arg(x, "edx"), _arg(x, "r8"), _arg(x, "rcx")
        if not (n and tb and hm): continue
        ids = {ida_bytes.get_qword(tb+16*i+8): idc.get_strlit_contents(ida_bytes.get_qword(tb+16*i)).decode() for i in range(n)}
        hand = {}
        for p in {ida_funcs.get_func(r).start_ea for r in idautils.DataRefsTo(hm) if ida_funcs.get_func(r)} - {ida_funcs.get_func(x).start_ea}:
            for c in idautils.CodeRefsTo(p, 0):
                try: txt = str(ida_hexrays.decompile(c))
                except Exception: continue
                for cid, h in re.findall(r"case (\d+):\s*\n\s*(?:[^\n]*?)(sub_[0-9A-F]+)\(", txt):
                    hand.setdefault(int(cid), h)
        out += [(hex(hm), i, nm, hand.get(i)) for i, nm in sorted(ids.items())]
    return out


# 1b. Switch scan near the parse function (AuthSrv pattern: parse fills a global record,
#     an other vtable slot of the same class calls the switch 1-3 calls deeper).
def switches():
    out = {}
    for fe in idautils.Functions():
        for h in idautils.FuncItems(fe):
            si = ida_nalt.get_switch_info(h)
            res = si and ida_xref.calc_switch_cases(h, si)
            if res: out[h] = (fe, {v for cv in res.cases for v in cv})
    return out


def _callees(fe):
    out = set()
    for h in idautils.FuncItems(fe):
        for r in idautils.CodeRefsFrom(h, 0):
            g = ida_funcs.get_func(r)
            if g and g.start_ea == r: out.add(r)
    return out


def _near(hm, depth=3):
    seeds = set()
    for r in idautils.DataRefsTo(hm):
        p = ida_funcs.get_func(r)
        if not p: continue
        for c in list(idautils.CodeRefsTo(p.start_ea, 0)) + [p.start_ea]:
            g = ida_funcs.get_func(c); seeds.add(g.start_ea)
            for vt in idautils.DataRefsTo(g.start_ea):   # vtable slot: add sibling slots
                for k in range(-4, 5):
                    q = ida_bytes.get_qword(vt + 8*k); s = ida_funcs.get_func(q)
                    if s and s.start_ea == q: seeds.add(q)
    seen, fr = set(seeds), set(seeds)
    for _ in range(depth):
        fr = {c for f in fr for c in _callees(f)} - seen; seen |= fr
    return seen


def handlers(rows=None):
    maps = {}
    for m, i, nm, hd in rows or commands(): maps.setdefault(m, {})[i] = nm
    sw = switches(); out = []
    for m, ids in maps.items():
        near = _near(int(m, 16)); best = (0, None)
        for h, (fe, vals) in sw.items():
            hit = len(vals & set(ids))
            if fe in near and hit >= 3 and hit >= 0.8*len(vals) and hit > best[0]: best = (hit, fe)
        if not best[1]: continue
        txt = str(ida_hexrays.decompile(best[1]))
        for grp, h in re.findall(r"((?:\s*case [0-9A-Fa-fx]+u?:\n)+)\s*[^\n]*?(sub_[0-9A-F]+|\w+)\(", txt):
            for v in re.findall(r"case ([0-9A-Fa-fx]+)u?:", grp):
                if int(v, 0) in ids: out.append((m, hex(best[1]), int(v, 0), ids[int(v, 0)], h))
    return out


def table():   # (map, id, name, handler or None)
    rows = commands(); sw = {(m, i): h for m, f, i, nm, h in handlers(rows)}
    return [(m, i, nm, hd or sw.get((m, i))) for m, i, nm, hd in rows]


# 2. Calls of a framework function with decoded arguments (numbers, wide strings, globals).
def _val(e):
    while e.op in (ida_hexrays.cot_cast, ida_hexrays.cot_ref): e = e.x
    if e.op == ida_hexrays.cot_num: return e.numval()
    if e.op == ida_hexrays.cot_idx and e.x.op == ida_hexrays.cot_obj and e.y.op == ida_hexrays.cot_num:
        p = ida_bytes.get_qword(e.x.obj_ea + 8 * e.y.numval())   # off_X[i] = string pointer table
        return _wstr(p) or hex(p)
    if e.op == ida_hexrays.cot_obj:
        return _wstr(e.obj_ea) or idc.get_name(e.obj_ea) or hex(e.obj_ea)
    return None


def hrcalls(name):
    tg = {idc.get_name_ea_simple(name)}
    tg |= {ida_funcs.get_func(x).start_ea for x in idautils.CodeRefsTo(min(tg), 0)
           if ida_funcs.get_func(x) and ida_funcs.get_func(x).end_ea - ida_funcs.get_func(x).start_ea <= 16}
    out = []
    for fe in {ida_funcs.get_func(x).start_ea for t in tg for x in idautils.CodeRefsTo(t, 0) if ida_funcs.get_func(x)} - tg:
        try: cf = ida_hexrays.decompile(fe)
        except Exception: continue
        class V(ida_hexrays.ctree_visitor_t):
            def visit_expr(s, e):
                if e.op == ida_hexrays.cot_call and e.x.op == ida_hexrays.cot_obj and e.x.obj_ea in tg:
                    out.append((hex(e.ea), idc.get_func_name(fe), [_val(a) for a in e.a]))
                return 0
        V(ida_hexrays.CV_FAST).apply_to(cf.body, None)
    return out


# 3. Config keys: (getter, call, func, args). IniGet*: [ini, section, key, index, default];
#    NetCfgInt: [ini, section (computed), key, default, flag] (see chapter 1.4).
def cfgkeys():
    return [(fn,) + r for fn in ("IniGetStr", "IniGetInt", "IniGetValues", "NetCfgInt", "NetCfgLookup") for r in hrcalls(fn)]


# 5. DB procs: (call, func, [writer, msg, logical db, schema, proc, flag]).
#    The proc builder is the small function that writes "SchemaName" and "ProcName".
def dbprocs():
    for s in idautils.Strings():
        if str(s) != "ProcName": continue
        for r in idautils.DataRefsTo(s.ea):
            f = ida_funcs.get_func(r)
            if f and f.end_ea - f.start_ea < 0x200:
                idc.set_name(f.start_ea, "DbProcBegin", idc.SN_NOWARN | idc.SN_FORCE)
                return hrcalls("DbProcBegin")
    return []


# 4. Timers: (call, func, notify, ms, flag, OnTimer if notify is a global object).
def timers():
    out = []
    for ea, fn, a in hrcalls("StsTimerSet"):
        o = idc.get_name_ea_simple(a[0]) if isinstance(a[0], str) else idc.BADADDR
        cb = ida_bytes.get_qword(ida_bytes.get_qword(o)) if o != idc.BADADDR else None
        out.append((ea, fn, a[0], a[1], a[2], cb and hex(cb)))
    return out
