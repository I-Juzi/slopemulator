# Match functions of a named build (A) to an unnamed build (B) from feat.py output.
# usage: python3 -I match.py A.jsonl B.jsonl OUT.json
import sys, json, collections

def load(p):
    F, V = {}, {}
    for line in open(p):
        r = json.loads(line)
        if 'vt' in r: V[r['vt']] = r
        else: F[r['ea']] = r
    return F, V

A, AV = load(sys.argv[1]); B, BV = load(sys.argv[2])
m = {}; rm = {}; how = {}
def ok_size(a, b):
    x, y = A[a]['size'], B[b]['size']
    return 0.5 <= (x + 8) / (y + 8) <= 2.0
def link(a, b, why):
    if a in m or b in rm: return False
    if why not in ('bytes', 'vtable'):
        sa, sb = set(A[a]['strs']), set(B[b]['strs'])
        if sa and sb and not (sa & sb): return False
        ca, cb = set(A[a]['consts']), set(B[b]['consts'])
        if len(ca) >= 3 and len(cb) >= 3 and len(ca & cb) * 4 < min(len(ca), len(cb)): return False
    m[a] = b; rm[b] = a; how[a] = why; return True
def informative(a): return not A[a]['name'].startswith(('sub_', 'nullsub', 'j_'))

def unique_round(keyf, why, cond=lambda a, b: True):
    ka, kb = collections.defaultdict(list), collections.defaultdict(list)
    for a, r in A.items():
        if a in m: continue
        k = keyf(r)
        if k is not None: ka[k].append(a)
    for b, r in B.items():
        if b in rm: continue
        k = keyf(r)
        if k is not None: kb[k].append(b)
    n = 0
    for k, la in ka.items():
        lb = kb.get(k)
        if len(la) == 1 and lb and len(lb) == 1 and cond(la[0], lb[0]):
            n += link(la[0], lb[0], why)
    return n

def stat(tag, n): print(f'{tag:28} +{n:7}  total {len(m):7} / A {len(A)} B {len(B)}', flush=True)

# R0: vtables with the same mangled name and slot count -> slots match by position
n = 0
for name, va in AV.items():
    vb = BV.get(name)
    if vb and len(va['slots']) == len(vb['slots']):
        for a, b in zip(va['slots'], vb['slots']):
            if a in A and b in B and ok_size(a, b): n += link(a, b, 'vtable')
stat('vtable slots', n)
# R1: exact masked bytes, unique
stat('masked bytes', unique_round(lambda r: (r['mask'], r['size']) if r['ninsn'] >= 6 else None, 'bytes'))
# R2: same string set, unique
stat('strings', unique_round(lambda r: tuple(sorted(set(r['strs']))) if r['strs'] else None, 'strings', ok_size))
# R3: opcode sequence + CFG shape, unique
stat('mnemonics+cfg', unique_round(lambda r: (r['mnem'], r['nb'], r['ne']) if r['ninsn'] >= 10 else None, 'mnem'))
# R4: constants set, unique
stat('constants', unique_round(lambda r: tuple(r['consts']) if len(r['consts']) >= 2 else None, 'consts', ok_size))

# R5: call-graph propagation
callers_a, callers_b = collections.defaultdict(list), collections.defaultdict(list)
for a, r in A.items():
    for c in r['calls']: callers_a[c].append(a)
for b, r in B.items():
    for c in r['calls']: callers_b[c].append(b)
def dedup(seq):
    s, o = set(), []
    for x in seq:
        if x not in s: s.add(x); o.append(x)
    return o
def sig(r): return (r['mnem'], r['nb'])
for it in range(30):
    n = 0
    for a, b in list(m.items()):
        for pos, la, lb in ((True, dedup(A[a]['calls']), dedup(B[b]['calls'])), (False, dedup(callers_a[a]), dedup(callers_b[b]))):
            ua = [x for x in la if x in A and x not in m]
            ub = [x for x in lb if x in B and x not in rm]
            if not ua or not ub: continue
            # same count and same order of unmatched callees -> positional (calls only)
            if pos and len(ua) == len(ub) and len(la) == len(lb):
                for x, y in zip(ua, ub):
                    if ok_size(x, y) and abs(A[x]['nb'] - B[y]['nb']) <= max(2, A[x]['nb'] // 4):
                        n += link(x, y, 'callpos')
                continue
            # otherwise unique feature within the neighbour set
            for kf in (lambda r: (r['mask'], r['size']), sig, lambda r: tuple(sorted(set(r['strs']))) or None):
                ka, kb = collections.defaultdict(list), collections.defaultdict(list)
                for x in ua:
                    if x not in m: ka[kf(A[x])].append(x)
                for y in ub:
                    if y not in rm: kb[kf(B[y])].append(y)
                for k, xs in ka.items():
                    if k is None: continue
                    ys = kb.get(k)
                    if len(xs) == 1 and ys and len(ys) == 1 and ok_size(xs[0], ys[0]):
                        n += link(xs[0], ys[0], 'callnb')
    stat(f'propagation {it}', n)
    if n == 0: break

# validation: names of matched pairs where B already has a meaningful (RTTI/library) name
agree = disagree = 0
for a, b in m.items():
    nb = B[b]['name']
    if nb.startswith(('sub_', 'j_', 'nullsub', 'unknown')): continue
    if nb == A[a]['name']: agree += 1
    else: disagree += 1
print('validation vs B names: agree', agree, 'differ', disagree)
c = collections.Counter(how.values()); print('by method', dict(c))
named = sum(1 for a in m if informative(a))
print('matched with real names', named)
json.dump([dict(a=a, b=b, name=A[a]['name'], how=how[a]) for a, b in m.items()], open(sys.argv[3], 'w'))
