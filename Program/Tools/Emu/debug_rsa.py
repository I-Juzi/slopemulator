"""Trace only generated RSA calls; never inspect the protected key."""
from collections import deque
from unicorn import UC_HOOK_CODE
from unicorn.x86_const import UC_X86_REG_RSP, UC_X86_REG_RIP
from emu381 import Emulator
from generate_381 import rsa_vectors

emu = Emulator()
recent = deque(maxlen=32)
def trace(uc, address, size, data):
    recent.append(hex(address))
emu.uc.hook_add(UC_HOOK_CODE, trace)
try:
    rsa_vectors(emu)
except Exception as exc:
    print(str(exc))
    print('Recent instruction addresses:', list(recent))
    rsp = emu.uc.reg_read(UC_X86_REG_RSP)
    print('Stack return addresses:', [hex(int.from_bytes(emu.read(rsp+i, 8), 'little')) for i in range(0, 64, 8)])
    raise
