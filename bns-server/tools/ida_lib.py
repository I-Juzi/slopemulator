# Shared IDA helpers for spec slices (IDA 9.5, idalib).
#
# Load once per database (state stays loaded in that database):
#   WT = "<worktree>"; exec(open(WT + "/tools/ida_lib.py").read())
#   print(boot(WT))          # StServers: fingerprint names + recipe.py
#
# All helpers return text or lists, filtered and short. Print what you need.
# Use these. Do not write your own copies. Add a local helper only if none fits.
import re, idc, idautils, ida_bytes, ida_funcs, ida_hexrays, ida_nalt, ida_name, ida_ua, ida_xref

_DECL = re.compile(r"^(__int64|__int128|__m128\w*|_BYTE|_WORD|_DWORD|_QWORD|_OWORD|int|char|unsigned|signed|"
                   r"const|void|bool|BOOL|CHAR|DWORD|LARGE_\w+|wchar_t|struct|float|double)\b[^(=]*;\s*(//.*)?$")
_VCALL = re.compile(r"\(\*\(\*?\(?[^()]*\(__fastcall \*\*?\)\([^)]*\)\)\(\*\((?:_QWORD \*)?\)?(\w+) \+ (\d+)(?:LL)?\)\)")


def boot(wt, stservers=True):
    """Set up one database: framework names and recipe.py. Return counts."""
    idc.SN_FORCE = ida_name.SN_FORCE
    if not stservers:
        return "ida_lib loaded"
    g = globals()
    exec(open(wt + "/tools/fn_fingerprint.py").read(), g)
    re_dir = wt + "/Docs/Platform/StServers/RE/"
    n1 = g["apply_names"](re_dir + "framework-funcs.tsv")
    n2 = g["apply_names"](re_dir + "framework-names.tsv")
    exec(open(re_dir + "recipe.py").read(), g)
    return "ida_lib + recipe loaded; names applied: funcs %d, names %d; %s" % (n1, n2, idc.get_input_file_path())


# ---------- addresses and names ----------
def ea_of(x):
    """Address from int, hex text or name."""
    if isinstance(x, int):
        return x
    if isinstance(x, str) and re.fullmatch(r"(0x)?[0-9A-Fa-f]{6,}", x):
        return int(x, 16)
    return idc.get_name_ea_simple(x)


def fs(x):
    """Start of the function that holds x, or None."""
    f = ida_funcs.get_func(ea_of(x))
    return f.start_ea if f else None


def nm(x):
    """'0x1400C5000 Name' (name only when not sub_)."""
    ea = ea_of(x)
    n = idc.get_name(ea) or ""
    return hex(ea) + ("" if not n or n.startswith("sub_") else " " + n)


# ---------- strings ----------
def sval(ea):
    """String literal at ea (narrow or wide), else None."""
    if not ea or ea == idc.BADADDR:
        return None
    t = idc.get_str_type(ea)
    if t is None or t < 0:
        t = ida_nalt.STRTYPE_C_16 if ida_bytes.get_byte(ea) and not ida_bytes.get_byte(ea + 1) else ida_nalt.STRTYPE_C
    s = ida_bytes.get_strlit_contents(ea, -1, t)
    if not s:
        return None
    s = s.decode("utf-8", "replace")
    return s if len(s) >= 2 and sum(32 <= ord(c) < 127 for c in s) >= 0.8 * len(s) else None


def strs(x, n=30, w=80):
    """Strings that the function of x uses, in order, no repeats."""
    f = ida_funcs.get_func(ea_of(x)); out = []
    for h in idautils.FuncItems(f.start_ea):
        for d in idautils.DataRefsFrom(h):
            s = sval(d)
            if s is None and ida_bytes.get_qword(d) and idc.get_name(d).startswith("off_"):
                s = sval(ida_bytes.get_qword(d))
            if s:
                out.append(s[:w])
    return list(dict.fromkeys(out))[:n]


def xstr(pat, n=25, w=80):
    """Strings that match pat, with the functions that use them. Text, first n rows + count."""
    rows = []
    for s in idautils.Strings():
        v = str(s)
        if re.search(pat, v):
            fns = sorted({nm(fs(r)) for r in idautils.DataRefsTo(s.ea) if fs(r)})
            rows.append("%X %s | %s" % (s.ea, v[:w], ", ".join(fns[:4]) + (" +%d" % (len(fns) - 4) if len(fns) > 4 else "")))
    return "%d match\n" % len(rows) + "\n".join(rows[:n])


# ---------- calls and xrefs ----------
def callers(x):
    """Functions that call or jump to x (thunks followed once)."""
    ea = ea_of(x); tg = {ea}
    for r in idautils.CodeRefsTo(ea, 0):
        f = ida_funcs.get_func(r)
        if f and f.end_ea - f.start_ea <= 16 and f.start_ea != ea:
            tg.add(f.start_ea)
    out = {fs(r) for t in tg for r in idautils.CodeRefsTo(t, 0) if fs(r)} - tg
    return [nm(a) for a in sorted(out)]


def callsites(x):
    """Call addresses to x, with the caller function."""
    ea = ea_of(x)
    return ["%X in %s" % (r, nm(fs(r))) for r in idautils.CodeRefsTo(ea, 0) if fs(r)]


def drefs(x):
    """Data refs to x: '%X in func' or '%X (data)'."""
    return ["%X in %s" % (r, nm(fs(r))) if fs(r) else "%X (data %s)" % (r, idc.get_name(r) or "") for r in idautils.DataRefsTo(ea_of(x))]


def callees(x):
    """Direct call targets of the function of x, in order, no repeats."""
    f = ida_funcs.get_func(ea_of(x)); out = []
    for h in idautils.FuncItems(f.start_ea):
        if idc.print_insn_mnem(h) == "call":
            t = idc.get_operand_value(h, 0)
            n = idc.get_name(t) if t else ""
            out.append(nm(t) if ida_funcs.get_func(t) else (n or idc.print_operand(h, 0)))
    return list(dict.fromkeys(out))


def arg(call_ea, reg, back=10):
    """Value put in reg (lea/mov) before the call at call_ea, else None."""
    h = call_ea
    for _ in range(back):
        h = idc.prev_head(h)
        if idc.print_insn_mnem(h) == "call":
            return None
        if idc.print_insn_mnem(h) in ("lea", "mov") and idc.print_operand(h, 0) == reg:
            return idc.get_operand_value(h, 1)
    return None


# ---------- one-line survey ----------
def summ(x, n=12):
    """One block per function: name, size, callers, callees, strings, constants."""
    ea = fs(x)
    if ea is None:
        return "%s: no function" % x
    f = ida_funcs.get_func(ea); consts = set()
    for h in idautils.FuncItems(ea):
        for i in (0, 1):
            if idc.get_operand_type(h, i) == ida_ua.o_imm:
                v = idc.get_operand_value(h, i)
                if 16 <= v < 0x100000000 and v not in (0xFFFFFFFF, 0xFF, 0xFFFF):
                    consts.add(v)
    cr = callers(ea); ce = callees(ea); ss = strs(ea, n, 50)
    return "\n".join([
        "== %s size %d" % (nm(ea), f.end_ea - f.start_ea),
        "  callers %d: %s" % (len(cr), ", ".join(cr[:n])),
        "  calls %d: %s" % (len(ce), ", ".join(c.replace("0x", "") for c in ce[:n])),
        "  strs %d: %s" % (len(ss), " | ".join(ss)),
        "  consts: %s" % ", ".join(hex(c) if c > 9999 else str(c) for c in sorted(consts)[:n])])


def survey(xs, n=8):
    """summ() for many functions in one call."""
    return "\n".join(summ(x, n) for x in xs)


# ---------- pseudocode ----------
def hr_text(x):
    try:
        return str(ida_hexrays.decompile(fs(x)))
    except Exception as e:
        return "ERR %s" % e


def hr(x, pat=r'\(|return|if |case |switch|"', n=40, w=140, skip=0):
    """Filtered pseudocode of x: no declarations, vcalls as V[obj+off], with line numbers.
    pat=None keeps every statement. skip skips the first matches (to page)."""
    t = hr_text(x).split("\n"); out = []
    for i, l in enumerate(t[1:], 1):
        s = l.strip()
        if not s or s in ("{", "}") or s.startswith("//") or _DECL.match(s):
            continue
        if pat and not re.search(pat, s):
            continue
        out.append("%d: %s" % (i, _VCALL.sub(r"V[\1+\2]", s)[:w]))
    head = "== %s lines %d, match %d" % (nm(fs(x) or ea_of(x)), len(t), len(out))
    return head + "\n" + "\n".join(out[skip:skip + n]) + ("\n... %d more" % (len(out) - skip - n) if len(out) > skip + n else "")


def hrs(xs, pat=r'\(|return|if |case |switch|"', n=20, w=130):
    """hr() for many functions in one call."""
    return "\n".join(hr(x, pat, n, w) for x in xs)


# ---------- tables ----------
def swmap(x):
    """Switch in the function of x: {case: (target, first call or None)}."""
    f = ida_funcs.get_func(ea_of(x)); out = {}
    for h in idautils.FuncItems(f.start_ea):
        si = ida_nalt.get_switch_info(h)
        if not si:
            continue
        res = ida_xref.calc_switch_cases(h, si)
        if not res:
            continue
        for i, t in enumerate(res.targets):
            ea, call = t, None
            for _ in range(20):
                m = idc.print_insn_mnem(ea)
                if m == "call":
                    call = idc.print_operand(ea, 0); break
                if m == "jmp" and idc.get_operand_type(ea, 0) == idc.o_near:
                    ea = idc.get_operand_value(ea, 0); continue
                if m in ("jmp", "retn"):
                    break
                ea = idc.next_head(ea)
            for c in res.cases[i]:
                out[c] = (hex(t), call)
    return out


def vslots(v, n=24):
    """Vtable at v: slot offset, function, first string of the function."""
    out = []
    for i in range(n):
        q = ida_bytes.get_qword(v + 8 * i); f = ida_funcs.get_func(q)
        if not f or f.start_ea != q:
            break
        s = strs(q, 1, 30)
        out.append("+%#x %s%s" % (8 * i, nm(q), " '%s'" % s[0] if s else ""))
    return out


def dis(x, n=20):
    """n instructions from x."""
    ea = ea_of(x); out = []
    for _ in range(n):
        out.append("%X %s" % (ea, idc.GetDisasm(ea))); ea = idc.next_head(ea)
    return "\n".join(out)
