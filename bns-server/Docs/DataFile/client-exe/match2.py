# Second-stage matcher: order anchors + gap alignment + call-graph fuzzy scoring.
# usage: python3 -I match2.py A.jsonl B.jsonl M_IN.json M_OUT.json
import sys, json, collections, bisect

def ld(p):
    D = {}
    for l in open(p):
        r = json.loads(l)
        if 'vt' not in r: D[r['ea']] = r
    return D
A = ld(sys.argv[1]); B = ld(sys.argv[2])
M = json.load(open(sys.argv[3]))
m = {e['a']: e['b'] for e in M}; rm = {b: a for a, b in m.items()}; how = {e['a']: e['how'] for e in M}
def norm(x):
    if '\\' in x and ('.cpp' in x.lower() or '.h' in x.lower() or 'depot' in x.lower() or ':' in x[:3]):
        return x.replace('/', '\\').lower().rsplit('\\', 1)[-1]
    return x
for r in list(A.values()) + list(B.values()):
    r['S'] = {norm(x) for x in r['strs']}; r['C'] = set(r['consts']); r['K'] = set(r['calls'])
callers_a, callers_b = collections.defaultdict(set), collections.defaultdict(set)
for a, r in A.items():
    for c in r['K']: callers_a[c].add(a)
for b, r in B.items():
    for c in r['K']: callers_b[c].add(b)

def jac(x, y): return len(x & y) / len(x | y) if x and y else None
def sim(a, b):
    x, y = A[a], B[b]
    sa, sb = x['size'], y['size']
    r = min(sa, sb) / max(sa, sb, 1)
    if r < 0.5: return -9
    s = 2 * r
    s += 1 - abs(x['nb'] - y['nb']) / max(x['nb'], y['nb'], 1)
    if x['mask'] == y['mask']: s += 5
    elif x['mnem'] == y['mnem']: s += 3
    j = jac(x['S'], y['S'])
    if j is not None: s += 4 * j if j > 0 else -4
    elif bool(x['S']) != bool(y['S']): s -= 1.5
    j = jac(x['C'], y['C'])
    if j is not None: s += 2 * j - (1 if j == 0 else 0)
    # call graph agreement through already matched neighbours
    ka = {m[c] for c in x['K'] if c in m}; kb = {c for c in y['K'] if c in rm}
    if ka or kb: s += 3 * (len(ka & kb) / max(len(ka), len(kb))) - (1 if not (ka & kb) else 0)
    ca = {m[c] for c in callers_a[a] if c in m}; cb = {c for c in callers_b[b] if c in rm}
    if ca or cb: s += 2 * (len(ca & cb) / max(len(ca), len(cb))) - (0.5 if not (ca & cb) else 0)
    return s

def link(a, b, why):
    if a in m or b in rm: return False
    if A[a]['S'] and B[b]['S'] and not (A[a]['S'] & B[b]['S']): return False
    m[a] = b; rm[b] = a; how[a] = why; return True
def stat(t, n): print(f'{t:26} +{n:6}  total {len(m):6} / B {len(B)}', flush=True)

As, Bs = sorted(A), sorted(B)
def anchors():
    # longest increasing subsequence of b over pairs sorted by a
    P = sorted(m.items()); tails = []; tidx = []; prev = [-1] * len(P)
    for i, (a, b) in enumerate(P):
        k = bisect.bisect_left(tails, b)
        if k == len(tails): tails.append(b); tidx.append(i)
        else: tails[k] = b; tidx[k] = i
        prev[i] = tidx[k - 1] if k else -1
    out = []; i = tidx[-1] if tidx else -1
    while i != -1: out.append(P[i]); i = prev[i]
    return out[::-1]

TH = 3.0
for rnd in range(6):
    total = 0
    # 1) gap alignment between ordered anchors
    an = anchors(); n = 0
    for (a0, b0), (a1, b1) in zip(an, an[1:]):
        ga = [x for x in As[bisect.bisect_right(As, a0):bisect.bisect_left(As, a1)] if x not in m]
        gb = [y for y in Bs[bisect.bisect_right(Bs, b0):bisect.bisect_left(Bs, b1)] if y not in rm]
        if not ga or not gb or len(ga) * len(gb) > 250000: continue
        R, C = len(ga), len(gb)
        S = [[sim(ga[i], gb[j]) for j in range(C)] for i in range(R)]
        D = [[0.0] * (C + 1) for _ in range(R + 1)]
        for i in range(R):
            Di, Dn, Si = D[i], D[i + 1], S[i]
            for j in range(C):
                v = Di[j] + (Si[j] - TH if Si[j] >= TH else -1e9)
                Dn[j + 1] = max(v, Di[j + 1], Dn[j])
        i, j = R, C
        while i and j:
            if D[i][j] == D[i - 1][j]: i -= 1
            elif D[i][j] == D[i][j - 1]: j -= 1
            else:
                n += link(ga[i - 1], gb[j - 1], 'gap'); i -= 1; j -= 1
    stat(f'round {rnd} gap align', n); total += n
    # 2) neighbour fuzzy: candidates are unmatched callees/callers of matched pairs
    cand = collections.defaultdict(set)
    for a, b in list(m.items()):
        for xa, xb in ((A[a]['K'], B[b]['K']), (callers_a[a], callers_b[b])):
            ua = [x for x in xa if x in A and x not in m]; ub = [y for y in xb if y in B and y not in rm]
            if len(ua) * len(ub) > 400: continue
            for x in ua:
                for y in ub: cand[x].add(y)
    best_b = {}
    for x, ys in cand.items():
        sc = sorted(((sim(x, y), y) for y in ys), reverse=True)
        if sc and sc[0][0] >= TH + 1 and (len(sc) == 1 or sc[0][0] - sc[1][0] >= 1.5):
            best_b[x] = sc[0]
    # mutual best
    rev = collections.defaultdict(list)
    for x, (s, y) in best_b.items(): rev[y].append((s, x))
    n = 0
    for y, xs in rev.items():
        xs.sort(reverse=True)
        if len(xs) == 1 or xs[0][0] - xs[1][0] >= 1.5: n += link(xs[0][1], y, 'fuzzy')
    stat(f'round {rnd} call fuzzy', n); total += n
    if total == 0: break

json.dump([dict(a=a, b=b, name=A[a]['name'], how=how[a], size=B[b]['size']) for a, b in m.items()], open(sys.argv[4], 'w'))
# checks
tot = collections.Counter(); bad = collections.Counter(); cnt = collections.Counter(how.values())
for a, b in m.items():
    sa, sb = A[a]['S'], B[b]['S']
    if sa and sb:
        tot[how[a]] += 1
        if not (sa & sb): bad[how[a]] += 1
for k in cnt: print(f'{k:8} {cnt[k]:6}  with strings {tot[k]:5}  no shared string {bad[k]:4} ({100*bad[k]/max(tot[k],1):.1f}%)')
print('matched', len(m), 'of B', len(B), f'({100*len(m)/len(B):.1f}%)')
