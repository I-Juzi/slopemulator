"""Probe original call boundaries with generated test data only."""
from emu381 import Emulator, u32
from generate_381 import make_rsa_base, mgf_call, rsa_material, PREVIOUS

emu = Emulator()
obj = make_rsa_base(emu)
emu.call(0x140069930, obj + 16, 1024)
case = next(c for c in PREVIOUS['cases'] if c['kind'] == 'oaep_encode' and c['case'] == 'k128_empty')
em = bytes.fromhex(case['output'])[::-1]
seed = bytes(a ^ b for a, b in zip(em[1:17], mgf_call(emu, obj + 48, em[17:], 16)))
db = bytes(a ^ b for a, b in zip(em[17:], mgf_call(emu, obj + 48, seed, 111)))
label = b'\xff' * 16
src, dst, size = emu.alloc(16, label), emu.alloc(16), emu.alloc(4, u32(16))
emu.call(0x14006F600, emu.alloc(16), src, 16, dst, size)
print('original OAEP recovered label hash:', db[:16].hex())
print('original digest of FF*16:', emu.read(dst, 16).hex())
print('recovered seed matches:', seed.hex() == case['seed'])
# Fresh generated RSA test key. This is only an ABI probe.
_, _, blob, private_blob = rsa_material()
ptr = emu.alloc(len(blob), blob)
start = len(emu.calls)
print('public setup result:', emu.call(0x1400668E0, obj, len(blob), ptr) & 255)
print('setup sentinel:', emu.calls[start:])
def trace():
    print('encrypt core args:', [hex(emu.arg(i)) for i in range(8)], flush=True)
emu.hooks[0x1400662A0] = ('observe encrypt ABI', trace)
message = b'381 RSA'
src, dst, size = emu.alloc(len(message), message), emu.alloc(256), emu.alloc(4, u32(256))
emu.set_random(bytes(range(16)))
print('encrypt wrapper result:', emu.call(0x140066A10, obj, src, len(message), dst, size) & 255)
print('output length:', emu.word(size))
private_obj = make_rsa_base(emu)
private_ptr = emu.alloc(len(private_blob), private_blob)
print('private setup:', emu.call(0x140066B50, private_obj, len(private_blob), private_ptr) & 255, flush=True)
def trace_decrypt():
    print('decrypt core args:', [hex(emu.arg(i)) for i in range(8)], flush=True)
emu.hooks[0x140066580] = ('observe decrypt ABI', trace_decrypt)
plain, plain_size = emu.alloc(256), emu.alloc(4, u32(256))
print('decrypt wrapper result:', emu.call(0x140066D50, private_obj, dst, emu.word(size), plain, plain_size) & 255)
print('decrypt output:', emu.read(plain, min(emu.word(plain_size), 256)).hex())
for name, address, data in [('public', 0x1400668E0, blob), ('private', 0x140066B50, private_blob)]:
    data_ptr = emu.alloc(len(data), data)
    for extra in [(1024, data_ptr, len(data)), (data_ptr, len(data), 1024),
                  (1024, len(data), data_ptr), (data_ptr, 1024, len(data)),
                  (len(data), data_ptr, 1024), (data_ptr, len(data), 0, 1024)]:
        candidate = make_rsa_base(emu)
        try:
            result = emu.call(address, candidate, *extra) & 255
            print(name, [hex(v) for v in extra], result, emu.calls[-1], flush=True)
        except RuntimeError as exc:
            print(name, [hex(v) for v in extra], str(exc), flush=True)
print('stubs:', emu.stub_counts)
