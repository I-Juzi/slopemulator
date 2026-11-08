"""Read-only 381 black-box runner. No host code from the PE is executed.

Run from the project root with bns-server/.venv/bin/python.
The built-in RSA range is skipped during file reads, then zeroed in Unicorn.
Only ABI, object setup and external dependencies are supplied by this runner.
"""
from pathlib import Path
import argparse
import json
import struct

import pefile
from unicorn import (
    Uc, UcError, UC_ARCH_X86, UC_MODE_64, UC_HOOK_CODE,
    UC_HOOK_MEM_INVALID, UC_HOOK_MEM_READ, UC_HOOK_MEM_WRITE,
)
from unicorn.x86_const import *

ROOT = Path(__file__).resolve().parents[3]
BINARY = ROOT / "ida/LobbyGate_2020/LobbyGate.exe"
VECTORS = ROOT / "bns-server/Program/Server/BloodLib/tests/vectors/emu_381.json"
TIMESTAMP = 0x5EB5328B
SECRET = 0x140077290
SECRET_SIZE = 264
BASE = 0x140000000
PAGE = 0x1000
STACK = 0x20000000
STACK_SIZE = 0x200000
HEAP = 0x30000000
HEAP_SIZE = 0x2000000
TEB = 0x40000000
TLS = TEB + 0x2000
TLS_DATA = TEB + 0x3000
STUBS = 0x50000000
SENTINEL = 0x51000000


def u32(n):
    return struct.pack("<I", n)


def u64(n):
    return struct.pack("<Q", n)


def sanitized_pe(path):
    """Never read the built-in key, including during pefile parsing."""
    with path.open("rb") as f:
        dos = f.read(64)
        if dos[:2] != b"MZ":
            raise RuntimeError("Not a PE file")
        peoff = struct.unpack_from("<I", dos, 0x3C)[0]
        if peoff > 0x1000:
            raise RuntimeError("Unexpected PE header offset")
        f.seek(peoff)
        nt = f.read(24)
        if nt[:4] != b"PE\0\0":
            raise RuntimeError("Invalid PE signature")
        timestamp = struct.unpack_from("<I", nt, 8)[0]
        if timestamp != TIMESTAMP:
            raise RuntimeError(f"Wrong PE timestamp: {timestamp:#x}; expected {TIMESTAMP:#x}")
        sections = struct.unpack_from("<H", nt, 6)[0]
        optional_size = struct.unpack_from("<H", nt, 20)[0]
        header_end = peoff + 24 + optional_size + sections * 40
        if header_end >= 0x10000:
            raise RuntimeError("Unexpected header size")
        f.seek(0)
        headers = f.read(header_end)
        pe = pefile.PE(data=headers, fast_load=True)
        if pe.OPTIONAL_HEADER.ImageBase != BASE or pe.FILE_HEADER.Machine != 0x8664:
            raise RuntimeError("Unexpected image base or machine")
        excluded = []
        for s in pe.sections:
            lo = max(SECRET - BASE, s.VirtualAddress)
            hi = min(SECRET - BASE + SECRET_SIZE, s.VirtualAddress + s.SizeOfRawData)
            if lo < hi:
                excluded.append((s.PointerToRawData + lo - s.VirtualAddress,
                                 s.PointerToRawData + hi - s.VirtualAddress))
        if len(excluded) != 1 or excluded[0][1] - excluded[0][0] != SECRET_SIZE:
            raise RuntimeError("Cannot locate the full forbidden range")
        raw_end = max(s.PointerToRawData + s.SizeOfRawData for s in pe.sections)
        raw = bytearray(raw_end)
        # Read headers and sections only. Never read an overlay or the key span.
        raw[:len(headers)] = headers
        for s in pe.sections:
            start, end = s.PointerToRawData, s.PointerToRawData + s.SizeOfRawData
            cursor = start
            for lo, hi in excluded:
                if cursor < lo < end:
                    f.seek(cursor)
                    raw[cursor:lo] = f.read(lo - cursor)
                if cursor < hi and lo < end:
                    cursor = max(cursor, hi)
            if cursor < end:
                f.seek(cursor)
                raw[cursor:end] = f.read(end - cursor)
    pe = pefile.PE(data=bytes(raw), fast_load=True)
    pe.parse_data_directories(directories=[pefile.DIRECTORY_ENTRY["IMAGE_DIRECTORY_ENTRY_IMPORT"]])
    return pe


class Emulator:
    def __init__(self, path=BINARY):
        self.pe = sanitized_pe(path)
        self.uc = Uc(UC_ARCH_X86, UC_MODE_64)
        self.uc.mem_map(BASE, (self.pe.OPTIONAL_HEADER.SizeOfImage + PAGE - 1) & -PAGE)
        self.uc.mem_write(BASE, self.pe.header)
        for s in self.pe.sections:
            self.uc.mem_write(BASE + s.VirtualAddress, s.get_data())
        # Mandatory overwrite, even though the bytes were never read from disk.
        self.uc.mem_write(SECRET, bytes(SECRET_SIZE))
        self.uc.mem_map(STACK, STACK_SIZE)
        self.uc.mem_map(HEAP, HEAP_SIZE)
        self.uc.mem_map(TEB, 0x10000)
        self.uc.mem_map(STUBS, 0x10000)
        self.uc.mem_map(SENTINEL, PAGE)
        self.uc.mem_write(SENTINEL, b"\xCC")
        self.uc.reg_write(UC_X86_REG_GS_BASE, TEB)
        self.uc.mem_write(TEB + 0x30, u64(TEB))
        self.uc.mem_write(TEB + 0x08, u64(STACK + STACK_SIZE))
        self.uc.mem_write(TEB + 0x10, u64(STACK))
        self.uc.mem_write(TEB + 0x58, u64(TLS))
        self.uc.mem_write(TLS, u64(TLS_DATA))
        self.next_alloc = HEAP
        self.imports = {}
        self.hooks = {}
        self.stub_counts = {}
        self.calls = []
        self.prng_bytes = b""
        self.prng_used = 0
        self.returned = False
        self.error = None
        self.instruction_count = 0
        for desc in getattr(self.pe, "DIRECTORY_ENTRY_IMPORT", []):
            dll = desc.dll.decode("ascii")
            for imp in desc.imports:
                name = imp.name.decode("ascii") if imp.name else f"ordinal:{imp.ordinal}"
                address = STUBS + 16 * len(self.imports)
                self.imports[address] = (dll, name)
                self.uc.mem_write(imp.address, u64(address))
                self.uc.mem_write(address, b"\xC3")
        # A fake TLS PRNG object supplies only generated bytes. The functions
        # under test, their digest, MGF, OAEP and arithmetic are not hooked.
        self.random_address = STUBS + 0xF000
        vtable = self.alloc(32)
        prng = self.alloc(128)
        self.uc.mem_write(vtable + 16, u64(self.random_address))
        self.uc.mem_write(prng + 24, u64(vtable))
        self.uc.mem_write(TLS_DATA + 48, u64(prng))
        self.uc.mem_write(self.random_address, b"\xC3")
        self.hooks[self.random_address] = ("controlled TLS PRNG virtual output", self._prng)
        self.uc.hook_add(UC_HOOK_CODE, self._code)
        self.uc.hook_add(UC_HOOK_MEM_INVALID, self._invalid)
        self.uc.hook_add(UC_HOOK_MEM_READ | UC_HOOK_MEM_WRITE, self._forbidden,
                         begin=SECRET, end=SECRET + SECRET_SIZE - 1)

    def set_random(self, stream):
        self.prng_bytes = bytes(stream)
        self.prng_used = 0

    def _prng(self):
        length, dst, size = self.arg(2), self.arg(3), self.arg(4)
        end = self.prng_used + length
        if end > len(self.prng_bytes) or self.word(size) < length:
            raise RuntimeError("Controlled PRNG stream exhausted or output too small")
        self.uc.mem_write(dst, self.prng_bytes[self.prng_used:end])
        self.uc.mem_write(size, u32(length))
        self.prng_used = end
        self.ret(1)

    def _forbidden(self, uc, access, address, size, value, data):
        self.error = "Original code accessed the forbidden built-in key range"
        uc.emu_stop()

    def _invalid(self, uc, access, address, size, value, data):
        self.error = (f"Unmapped/protected access {access} at {address:#x}, size {size}, "
                      f"RIP {uc.reg_read(UC_X86_REG_RIP):#x}")
        return False

    def alloc(self, size, data=None):
        address = (self.next_alloc + 15) & -16
        self.next_alloc = address + max(size, 16)
        if self.next_alloc > HEAP + HEAP_SIZE:
            raise RuntimeError("Emulator heap exhausted")
        self.uc.mem_write(address, bytes(max(size, 16)))
        if data is not None:
            self.uc.mem_write(address, bytes(data))
        return address

    def read(self, address, size):
        if address < SECRET + SECRET_SIZE and address + size > SECRET:
            raise RuntimeError("Read of forbidden image range")
        return bytes(self.uc.mem_read(address, size))

    def word(self, address):
        return struct.unpack("<I", self.read(address, 4))[0]

    def arg(self, index):
        if index < 4:
            return self.uc.reg_read([UC_X86_REG_RCX, UC_X86_REG_RDX,
                                     UC_X86_REG_R8, UC_X86_REG_R9][index])
        return struct.unpack("<Q", self.read(self.uc.reg_read(UC_X86_REG_RSP)
                                            + 0x28 + 8 * (index - 4), 8))[0]

    def ret(self, result=1):
        rsp = self.uc.reg_read(UC_X86_REG_RSP)
        target = struct.unpack("<Q", self.read(rsp, 8))[0]
        self.uc.reg_write(UC_X86_REG_RAX, result & ((1 << 64) - 1))
        self.uc.reg_write(UC_X86_REG_RSP, rsp + 8)
        self.uc.reg_write(UC_X86_REG_RIP, target)

    def _import(self, dll, name):
        args = [self.arg(i) for i in range(4)]
        a, b, c, d = args
        if name in ("memcpy", "memmove"):
            self.uc.mem_write(a, self.read(b, c))
            return a
        if name == "memset":
            self.uc.mem_write(a, bytes([b & 255]) * c)
            return a
        if name == "memcmp":
            x, y = self.read(a, c), self.read(b, c)
            return (x > y) - (x < y)
        if name == "GetProcessHeap":
            return 1
        if name in ("HeapAlloc", "HeapReAlloc"):
            if name == "HeapReAlloc":
                raise RuntimeError("HeapReAlloc needs an explicit implementation")
            return self.alloc(c)
        if name == "HeapFree":
            return 1
        if name in ("malloc", "calloc"):
            return self.alloc(a if name == "malloc" else a * b)
        if name == "free":
            return 0
        raise RuntimeError(f"Unknown import called: {dll}!{name}")

    def _code(self, uc, address, size, data):
        self.instruction_count += 1
        if address == SENTINEL:
            self.returned = True
            uc.emu_stop()
            return
        try:
            if address in self.hooks:
                name, callback = self.hooks[address]
                self.stub_counts[name] = self.stub_counts.get(name, 0) + 1
                callback()
            elif address in self.imports:
                dll, name = self.imports[address]
                label = f"{dll}!{name}"
                self.stub_counts[label] = self.stub_counts.get(label, 0) + 1
                self.ret(self._import(dll, name))
        except Exception as exc:
            self.error = str(exc)
            uc.emu_stop()

    def call(self, address, *args, count=100_000_000):
        # Entry RSP is 8 mod 16. Reserve shadow space plus stack arguments.
        rsp = STACK + STACK_SIZE - 0x108
        self.uc.mem_write(rsp, bytes(0x100))
        self.uc.mem_write(rsp, u64(SENTINEL))
        for reg in [UC_X86_REG_RAX, UC_X86_REG_RBX, UC_X86_REG_RBP,
                    UC_X86_REG_RSI, UC_X86_REG_RDI, UC_X86_REG_R12,
                    UC_X86_REG_R13, UC_X86_REG_R14, UC_X86_REG_R15]:
            self.uc.reg_write(reg, 0)
        for i, reg in enumerate([UC_X86_REG_RCX, UC_X86_REG_RDX,
                                UC_X86_REG_R8, UC_X86_REG_R9]):
            self.uc.reg_write(reg, args[i] if i < len(args) else 0)
        for i, value in enumerate(args[4:]):
            self.uc.mem_write(rsp + 0x28 + i * 8, u64(value))
        self.uc.reg_write(UC_X86_REG_RSP, rsp)
        self.uc.reg_write(UC_X86_REG_EFLAGS, 0x202)
        self.returned, self.error = False, None
        before = self.instruction_count
        try:
            self.uc.emu_start(address, 0, count=count)
        except UcError as exc:
            raise RuntimeError(self.error or f"Unicorn: {exc}; RIP {self.uc.reg_read(UC_X86_REG_RIP):#x}") from exc
        if self.error:
            raise RuntimeError(self.error)
        if not self.returned:
            raise RuntimeError(f"{address:#x} did not return through the sentinel; RIP {self.uc.reg_read(UC_X86_REG_RIP):#x}")
        self.calls.append({"address": hex(address), "sentinel": True,
                           "instructions": self.instruction_count - before})
        return self.uc.reg_read(UC_X86_REG_RAX)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--imports", action="store_true")
    args = parser.parse_args()
    emu = Emulator()
    if args.imports:
        for dll, name in emu.imports.values():
            print(f"{dll}!{name}")


if __name__ == "__main__":
    main()
