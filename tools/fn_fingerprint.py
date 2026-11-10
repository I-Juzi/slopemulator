"""Function fingerprints for the StServers binaries (shared framework vs service code).

All StServers programs link the same framework. A fingerprint hashes each function's
instruction stream with address operands masked, so the same framework function gets
the same hash in every binary.

Inside IDA (MCP execute_python), a slice runs:
    exec(open("<repo>/tools/ida/fn_fingerprint.py").read())
    rows = service_funcs("<repo>/Docs/Platform/StServers/RE/framework-funcs.tsv")
    # rows: (ea, name, size) of the functions that are NOT framework code

Headless batch (no tokens), with the Python that has `idapro`:
    python fn_fingerprint.py batch <out_dir> <binary or .i64> ...
    python fn_fingerprint.py framework <out_dir> <framework.tsv> [names_dir]   # plain Python, no IDA

To import the framework names into the open database: apply_names("<framework.tsv>").
"""
import hashlib, os, sys


def _ida():
    import ida_funcs, ida_ua, ida_bytes, idautils, idc, ida_ida  # noqa: F401
    return ida_funcs, ida_ua, ida_bytes, idautils, idc, ida_ida


_IMPORTS = None


def _imports():
    global _IMPORTS
    if _IMPORTS is None:
        import ida_nalt
        _IMPORTS = {}
        for i in range(ida_nalt.get_import_module_qty()):
            def cb(ea, name, ordinal):
                _IMPORTS[ea] = name or f"#{ordinal}"
                return True
            ida_nalt.enum_import_names(i, cb)
    return _IMPORTS


def _addr_token(addr):
    """Masked address -> stable token: string text or import name, else 'A'.
    Strings and imports are the same in every binary, so framework code still matches,
    and service code that differs only in a command name or string does not."""
    import ida_bytes, idc
    imp = _imports().get(addr)
    if imp:
        return ("I", imp)
    if ida_bytes.is_strlit(ida_bytes.get_flags(addr)):
        b = idc.get_strlit_contents(addr, -1, idc.get_str_type(addr))
        if b:
            return ("S", b[:80])
    return "A"


def fingerprint(f):
    ida_funcs, ida_ua, ida_bytes, idautils, idc, ida_ida = _ida()
    lo, hi = ida_ida.inf_get_min_ea(), ida_ida.inf_get_max_ea()
    h = hashlib.sha1()
    n = 0
    for ea in idautils.FuncItems(f.start_ea):
        insn = ida_ua.insn_t()
        if not ida_ua.decode_insn(insn, ea):
            continue
        n += 1
        parts = [insn.itype]
        for op in insn.ops:
            t = op.type
            if t == ida_ua.o_void:
                break
            if t == ida_ua.o_mem:
                parts.append((t, _addr_token(op.addr)))
            elif t in (ida_ua.o_near, ida_ua.o_far):
                parts.append((t,))
            elif t == ida_ua.o_imm:
                parts.append((t, _addr_token(op.value)) if lo <= op.value < hi else (t, op.value))
            elif t == ida_ua.o_displ:
                parts.append((t, op.phrase, "A") if lo <= op.addr < hi else (t, op.phrase, op.addr))
            else:
                parts.append((t, op.reg, op.phrase))
        h.update(repr(parts).encode())
    return h.hexdigest()[:16], n


def all_funcs():
    import ida_funcs, idautils
    out = []
    for ea in idautils.Functions():
        f = ida_funcs.get_func(ea)
        fp, n = fingerprint(f)
        out.append((fp, ea, f.end_ea - f.start_ea, n, ida_funcs.get_func_name(ea)))
    return out


def load_framework(path):
    with open(path) as fh:
        next(fh)
        return {l.split("\t")[0] for l in fh if l.strip()}


def service_funcs(framework_tsv, min_insns=1):
    """Functions of the open database that are not framework code: (ea, name, size)."""
    fw = load_framework(framework_tsv)
    return [(hex(ea), name, size) for fp, ea, size, n, name in all_funcs() if fp not in fw and n >= min_insns]


def apply_names(framework_tsv):
    """Rename sub_ functions whose hash has a framework name. Return the count."""
    import ida_funcs, idc
    names = {}
    with open(framework_tsv) as fh:
        next(fh)
        for l in fh:
            h, _, nm = l.rstrip("\n").split("\t")[:3]
            if nm and not nm.startswith(("sub_", "nullsub_", "j_")):
                names[h] = nm
    n = 0
    for fp, ea, size, k, name in all_funcs():
        if name.startswith("sub_") and fp in names:
            if idc.set_name(ea, names[fp], idc.SN_NOWARN | idc.SN_NOCHECK | idc.SN_FORCE):
                n += 1
    return n


# ---------- headless ----------
def batch(out_dir, paths):
    global _IMPORTS
    import idapro
    import ida_auto
    os.makedirs(out_dir, exist_ok=True)
    for p in paths:
        name = os.path.basename(p).split(".")[0]
        out = os.path.join(out_dir, name + ".tsv")
        if os.path.exists(out):
            continue
        src = p + ".i64" if os.path.exists(p + ".i64") and not p.endswith(".i64") else p
        if idapro.open_database(src, True):
            print(f"open failed: {src}", flush=True)
            continue
        ida_auto.auto_wait()
        _IMPORTS = None
        rows = all_funcs()
        with open(out + ".tmp", "w") as fh:
            fh.write("hash\tea\tsize\tinsns\tname\n")
            for fp, ea, size, n, nm in rows:
                fh.write(f"{fp}\t{ea:#x}\t{size}\t{n}\t{nm}\n")
        os.replace(out + ".tmp", out)
        idapro.close_database(True)
        print(f"{name}: {len(rows)} functions", flush=True)


def framework(out_dir, out_tsv, share=0.5, template="HelloSrv", names_dir=None):
    """Framework = in the template binary, or in at least `share` of all binaries.
    names_dir: dumps of analysed databases; only their names are used, not their counts."""
    tabs = {}
    for fn in sorted(os.listdir(out_dir)):
        if fn.endswith(".tsv"):
            with open(os.path.join(out_dir, fn)) as fh:
                next(fh)
                tabs[fn[:-4]] = [l.rstrip("\n").split("\t") for l in fh if l.strip()]
    count, sample = {}, {}
    for rows in tabs.values():
        for h in {r[0] for r in rows}:
            count[h] = count.get(h, 0) + 1
        for r in rows:
            if not r[4].startswith("sub_"):
                sample.setdefault(r[0], r[4])
            sample.setdefault(r[0], r[4])
    if names_dir and os.path.isdir(names_dir):
        for fn in sorted(os.listdir(names_dir)):
            if fn.endswith(".tsv"):
                with open(os.path.join(names_dir, fn)) as fh:
                    next(fh)
                    for l in fh:
                        r = l.rstrip("\n").split("\t")
                        if len(r) > 4 and not r[4].startswith(("sub_", "nullsub_", "j_")) and sample.get(r[0], "sub_").startswith("sub_"):
                            sample[r[0]] = r[4]
    tmpl = {r[0] for r in tabs.get(template, [])}
    fw = {h for h, c in count.items() if h in tmpl or c >= share * len(tabs)}
    with open(out_tsv, "w") as fh:
        fh.write("hash\tbinaries\tsample_name\n")
        for h in sorted(fw, key=lambda h: (-count[h], h)):
            fh.write(f"{h}\t{count[h]}\t{sample[h]}\n")
    print(f"framework: {len(fw)} hashes from {len(tabs)} binaries")
    print("binary\tfunctions\tservice_funcs\tservice_bytes")
    for b, rows in sorted(tabs.items()):
        own = [r for r in rows if r[0] not in fw]
        print(f"{b}\t{len(rows)}\t{len(own)}\t{sum(int(r[2]) for r in own)}")


if __name__ == "__main__":
    if sys.argv[1] == "batch":
        batch(sys.argv[2], sys.argv[3:])
    elif sys.argv[1] == "framework":
        framework(sys.argv[2], sys.argv[3], names_dir=sys.argv[4] if len(sys.argv) > 4 else None)
