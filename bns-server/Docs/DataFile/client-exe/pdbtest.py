import sys, os, json, random
sys.path.insert(0, os.path.expanduser('~/ida-pro-9.5/idalib/python'))
import idapro
db, pdb, mj = sys.argv[1:4]
idapro.open_database(db, False)
import idautils, ida_name, ida_netnode, ida_loader, ida_nalt, ida_auto, ida_funcs, ida_typeinf
def named(): return sum(1 for f in idautils.Functions() if not ida_name.get_name(f).startswith(('sub_','nullsub','j_')))
print('before', named(), flush=True)
n = ida_netnode.netnode('$ pdb', 0, True)
n.altset(0, ida_nalt.get_imagebase()); n.supset(0, pdb)
print('plugin', ida_loader.load_and_run_plugin('pdb', 3), flush=True)
ida_auto.auto_wait()
print('after', named(), flush=True)
M = json.load(open(mj)); random.seed(1)
ok = typed = 0
for e in M:
    nm = ida_name.get_name(e['b'])
    if not nm.startswith(('sub_','nullsub','j_')): ok += 1
    if ida_typeinf.tinfo_t() and ida_nalt.get_tinfo(ida_typeinf.tinfo_t(), e['b']): typed += 1
print('matched addrs named', ok, 'of', len(M), 'typed', typed)
for e in random.sample(M, 8):
    print(hex(e['b']), e['how'], ida_name.get_name(e['b'])[:70], '| demangled:', (ida_name.demangle_name(ida_name.get_name(e['b']), 0) or '')[:60])
idapro.close_database(True)
