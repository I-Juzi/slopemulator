"""Compare two TableDefs folders field by field. Prints counts only.
Usage: python3 -I diffdefs.py REF_DIR NEW_DIR [--same-version]"""
import collections, glob, json, os, sys

ref = {}
for f in glob.glob(os.path.join(sys.argv[1], '*.json')):
    j = json.load(open(f)); ref[j['name']] = j
new = {}
for f in glob.glob(os.path.join(sys.argv[2], '*.json')):
    j = json.load(open(f)); new[j['name']] = j
same_only = '--same-version' in sys.argv
C = collections.Counter(); ex = {}


def cmp(path, a, b):
    if isinstance(a, dict) and isinstance(b, dict):
        for k in a.keys() | b.keys():
            cmp(path + (k,), a.get(k, '<missing>'), b.get(k, '<missing>'))
    elif isinstance(a, list) and isinstance(b, list) and len(a) == len(b):
        for x, y in zip(a, b):
            cmp(path + ('[]',), x, y)
    else:
        key = '.'.join(p for p in path)
        if a == b or (isinstance(a, float) and isinstance(b, float) and abs(a - b) <= abs(a) * 1e-6):
            C[(key, 'ok')] += 1
        else:
            C[(key, 'DIFF')] += 1
            ex.setdefault(key, (path, repr(a)[:60], repr(b)[:60]))


n = 0
for name, r in ref.items():
    b = new.get(name)
    if b is None:
        C[('table', 'missing')] += 1; continue
    if same_only and (r['major_ver'], r['minor_ver']) != (b['major_ver'], b['minor_ver']):
        continue
    n += 1
    r = dict(r); b = dict(b); r.pop('type', None); b.pop('type', None)
    cmp((name,), {k: v for k, v in r.items()}, b)

print('tables compared', n)
agg = collections.Counter()
for (k, s), v in C.items():
    agg[(k.split('.', 1)[1] if '.' in k else k, s)] += v
for (k, s), v in sorted(agg.items()):
    if s != 'ok':
        print('%-45s %s %d' % (k, s, v))
print('fields ok', sum(v for (k, s), v in agg.items() if s == 'ok'))
for k, (p, a, b) in list(ex.items())[:15]:
    print('  e.g.', '.'.join(p)[:70], a, '->', b)
