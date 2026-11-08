import struct, sys
sys.path.insert(0, '.')
exec(open('findtd.py').read().split("for name in")[0])
p = struct.pack('<I', 0x1453260); j = -1
while True:
    j = d.find(p, j + 1)
    if j < 0: break
    print('ptr to td at va', hex(off2va(j)), [s[0] for s in secs if s[3] <= j < s[3] + s[4]])
    # walk back/forward over consecutive pointers whose target has a UTF-16 name pointer
    def is_td(v):
        o = va2off(v)
        if o is None: return False
        nv = struct.unpack_from('<I', d, o)[0]; no = va2off(nv)
        return no is not None and d[no:no+2] != b'\0\0' and d[no+1] == 0
    s = j
    while s >= 4 and (struct.unpack_from('<I', d, s-4)[0] == 0 or is_td(struct.unpack_from('<I', d, s-4)[0])): s -= 4
    e = j
    while is_td(struct.unpack_from('<I', d, e)[0]) or struct.unpack_from('<I', d, e)[0] == 0 and is_td(struct.unpack_from('<I', d, e+4)[0]): e += 4
    print('  run from', hex(off2va(s)), 'to', hex(off2va(e)), 'slots', (e - s)//4)
