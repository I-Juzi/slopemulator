"""Generate 381 oracle vectors, one item at a time. No crypto is translated.

Commands (from the project root):
  lobbygate-rs/.venv/bin/python lobbygate-rs/tools/emu/generate_381.py --through 1
  cargo test --manifest-path lobbygate-rs/Cargo.toml --test emu_381

Only after that item passes, raise --through. All data in these vectors is public
synthetic test material. A PRNG stream is supplied at its virtual-call boundary.
"""
import argparse
import json
import secrets

from emu381 import Emulator, VECTORS, TIMESTAMP, STUBS, TLS_DATA, u32, u64


PREVIOUS = json.loads(VECTORS.read_text()) if VECTORS.exists() else {"cases": []}


def generated(kind, case, field, length):
    for old in PREVIOUS["cases"]:
        if old["kind"] == kind and old["case"] == case and field in old:
            value = bytes.fromhex(old[field])
            if len(value) == length:
                return value
    return secrets.token_bytes(length)


def checked_output(emu, address, capacity, length, input_bytes):
    if not 0 <= length <= capacity:
        raise RuntimeError(f"Invalid original output length {length}")
    output = emu.read(address, length)
    if output == input_bytes:
        raise RuntimeError("Original output is a copy of input")
    if length and output == bytes([0xA5]) * length:
        raise RuntimeError("Original did not replace the output poison")
    return output


def record(emu, start, kind, case, source, **values):
    calls = emu.calls[start:]
    if not any(c["address"] == source and c["sentinel"] for c in calls):
        raise RuntimeError(f"No sentinel return from {source}")
    return dict(kind=kind, case=case, source=source, calls=calls, **values)


def hash_vectors(emu):
    cases = [b"", b"abc", bytes(range(256)) + b"381 multi-block"]
    output = []
    obj = emu.alloc(16)
    for i, message in enumerate(cases):
        src = emu.alloc(len(message), message)
        dst = emu.alloc(16, b"\xA5" * 16)
        size = emu.alloc(4, u32(16))
        start = len(emu.calls)
        result = emu.call(0x14006F600, obj, src, len(message), dst, size) & 255
        if result != 1 or emu.word(size) != 16:
            raise RuntimeError(f"RIPEMD failed: return={result}, length={emu.word(size)}")
        digest = checked_output(emu, dst, 16, 16, message)
        output.append(record(emu, start, "ripemd128", ["empty", "short", "multi_block"][i],
                             "0x14006f600", input=message.hex(), output=digest.hex(), result=result))
    return output


def make_rsa_base(emu):
    obj = emu.alloc(2048)
    emu.call(0x1400661A0, obj)
    return obj


def mgf_call(emu, mgf, data, length):
    src = emu.alloc(len(data), data)
    dst = emu.alloc(length, b"\xA5" * length)
    result = emu.call(0x140069750, mgf, src, len(data), dst, length) & 255
    if result != 1:
        raise RuntimeError(f"MGF failed: {result}")
    return checked_output(emu, dst, length, length, data)


def mgf_vectors(emu):
    obj = make_rsa_base(emu)
    if emu.call(0x140069930, obj + 16, 1024) & 255 != 1:
        raise RuntimeError("OAEP setup failed while preparing MGF")
    output = []
    # The production Rust MGF is private. Rust probes it through oaep_encode.
    for length in [7, 16, 47]:
        seed = generated("mgf", f"length_{length}", "input", 16)
        start = len(emu.calls)
        mask = mgf_call(emu, obj + 48, seed, length)
        output.append(record(emu, start, "mgf", f"length_{length}", "0x140069750",
                             input=seed.hex(), length=length, output=mask.hex(), result=1))
    return output


def oaep_vectors(emu):
    cases = []
    for k in [128, 64]:
        obj = make_rsa_base(emu)
        oaep, mgf = obj + 16, obj + 48
        if emu.call(0x140069930, oaep, k * 8) & 255 != 1:
            raise RuntimeError("OAEP setup failed")
        encoded = {}
        for label, message in [("empty", b""), ("short", b"381 OAEP"),
                               ("capacity", bytes(range(k - 34))),
                               ("over_capacity", bytes(range(k - 33)))]:
            name = f"k{k}_{label}"
            seed = generated("oaep_encode", name, "seed", 16)
            src = emu.alloc(len(message), message)
            dst = emu.alloc(k, b"\xA5" * k)
            size = emu.alloc(4, u32(k))
            emu.set_random(seed)
            start = len(emu.calls)
            result = emu.call(0x1400699D0, oaep, src, len(message), dst, size, 0, 0) & 255
            expected_ok = len(message) <= k - 34
            if bool(result) != expected_ok:
                raise RuntimeError(f"Unexpected OAEP encode result: {name}, {result}")
            output = checked_output(emu, dst, k, emu.word(size), message) if result else b""
            if emu.prng_used != (16 if result else 0):
                raise RuntimeError("Unexpected OAEP random consumption")
            cases.append(record(emu, start, "oaep_encode", name, "0x1400699d0",
                                k=k, input=message.hex(), seed=seed.hex(), output=output.hex(),
                                result=result, output_length=emu.word(size), random_used=emu.prng_used))
            if result:
                encoded[label] = output
        decode_inputs = [(label, block) for label, block in encoded.items()]
        bad00 = bytearray(encoded["short"])
        bad00[-1] = 0xA7
        decode_inputs.append(("bad_00", bytes(bad00)))
        # Compose malformed blocks from the spec using the original MGF as an
        # oracle. There is no Python implementation of the original digest.
        seed = generated("oaep_encode", f"k{k}_short", "seed", 16)
        for label in ["bad_label_hash", "no_01_delimiter"]:
            db = bytearray(k - 17)
            db[:16] = b"\x42" * 16
            if label == "bad_label_hash":
                db[-9:] = b"\x01381 OAEP"
            mask = mgf_call(emu, mgf, seed, len(db))
            masked_db = bytes(a ^ b for a, b in zip(db, mask))
            mask = mgf_call(emu, mgf, masked_db, 16)
            masked_seed = bytes(a ^ b for a, b in zip(seed, mask))
            decode_inputs.append((label, (b"\0" + masked_seed + masked_db)[::-1]))
        for label, block in decode_inputs:
            name = f"k{k}_{label}"
            src = emu.alloc(k, block)
            dst = emu.alloc(k, b"\xA5" * k)
            size = emu.alloc(4, u32(k))
            start = len(emu.calls)
            result = emu.call(0x140069D30, oaep, src, k, dst, size) & 255
            if bool(result) != (label != "no_01_delimiter"):
                raise RuntimeError(f"Unexpected OAEP decode result: {name}, {result}")
            output = checked_output(emu, dst, k, emu.word(size), block) if result else b""
            if not result and emu.read(dst, k) != b"\xA5" * k:
                raise RuntimeError("Failed OAEP decode changed the output")
            cases.append(record(emu, start, "oaep_decode", name, "0x140069d30",
                                k=k, input=block.hex(), output=output.hex(), result=result,
                                output_length=emu.word(size)))
    return cases


def rsa_material():
    """Generate a fresh 1024-bit test key with Miller-Rabin and pow()."""
    def prime():
        while True:
            n = secrets.randbits(512) | (1 << 511) | 1
            if any(n % p == 0 for p in [3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37]):
                continue
            d, s = n - 1, 0
            while d % 2 == 0:
                d //= 2
                s += 1
            for _ in range(40):
                x = pow(secrets.randbelow(n - 3) + 2, d, n)
                if x in (1, n - 1):
                    continue
                for _ in range(s - 1):
                    x = pow(x, 2, n)
                    if x == n - 1:
                        break
                else:
                    break
            else:
                return n
    while True:
        p, q = prime(), prime()
        n, phi = p * q, (p - 1) * (q - 1)
        if p != q and n.bit_length() == 1024 and phi % 65537:
            d = pow(65537, -1, phi)
            public = u32(128) + n.to_bytes(128, 'little')
            private = public + u32(128) + d.to_bytes(128, 'little')
            return n, d, public, private


def rsa_vectors(emu):
    n, d, public_blob, private_blob = rsa_material()
    cases = []
    setup_start = len(emu.calls)
    public, private = make_rsa_base(emu), make_rsa_base(emu)
    for obj, address, blob in [(public, 0x1400668E0, public_blob),
                               (private, 0x140066B50, private_blob)]:
        ptr = emu.alloc(len(blob), blob)
        if emu.call(address, obj, ptr, len(blob), 1024) & 255 != 1:
            raise RuntimeError('Generated RSA key setup failed')
    setup_calls = emu.calls[setup_start:]
    ciphertexts = []
    for name, message in [('short', b'381 RSA'), ('multi_chunk', bytes(range(201)))]:
        chunks = (len(message) + 93) // 94
        stream = generated('rsa_encrypt', name, 'seed', 16 * chunks)
        src, dst = emu.alloc(len(message), message), emu.alloc(chunks * 128, b'\xA5' * (chunks * 128))
        size = emu.alloc(4, u32(chunks * 128))
        emu.set_random(stream)
        start = len(emu.calls)
        result = emu.call(0x140066A10, public, src, len(message), dst, size) & 255
        if result != 1 or emu.prng_used != len(stream):
            raise RuntimeError(f'RSA encrypt failed: {name}, {result}')
        output = checked_output(emu, dst, chunks * 128, emu.word(size), message)
        case = record(emu, start, 'rsa_encrypt', name, '0x140066a10',
                      input=message.hex(), output=output.hex(), seed=stream.hex(),
                      public_blob=public_blob.hex(), private_blob=private_blob.hex(),
                      result=result, random_used=emu.prng_used, output_length=emu.word(size),
                      setup_calls=setup_calls)
        cases.append(case)
        ciphertexts.append((name, output, message))
    # The zero-block bad-OAEP path remains open: no sentinel return (RIP=0).
    # Do not discard the valid RSA vectors when that path fails.
    ciphertexts.append(('incomplete', ciphertexts[0][1][:-1], None))
    for name, ciphertext, message in ciphertexts:
        src, dst = emu.alloc(len(ciphertext), ciphertext), emu.alloc(512, b'\xA5' * 512)
        size = emu.alloc(4, u32(512))
        before = emu.read(private, 2048)
        start = len(emu.calls)
        result = emu.call(0x140066D50, private, src, len(ciphertext), dst, size, count=1_000_000_000) & 255
        print(f'RSA decrypt {name}: sentinel, return={result}', flush=True)
        after = emu.read(private, 2048)
        changes = {hex(i): int.from_bytes(after[i:i+4], 'little') for i in range(0, 2048, 4)
                   if before[i:i+4] != after[i:i+4]}
        if bool(result) != (message is not None):
            raise RuntimeError(f'RSA decrypt failed: {name}, {result}, changes={changes}')
        output = checked_output(emu, dst, 512, emu.word(size), ciphertext) if result else b''
        if result and output != message:
            raise RuntimeError(f'RSA plaintext differs: {name}')
        if name == 'bad_oaep' and 17 not in changes.values():
            raise RuntimeError(f'RSA OAEP error 17 absent: {changes}')
        cases.append(record(emu, start, 'rsa_decrypt', name, '0x140066d50',
                            input=ciphertext.hex(), output=output.hex(), result=result,
                            public_blob=public_blob.hex(), private_blob=private_blob.hex(),
                            output_length=emu.word(size), object_changes=changes,
                            setup_calls=setup_calls))
    return cases


def rsa_bad_oaep_vector(emu, cases):
    """Reject a nonzero RSA preimage with the verified missing-delimiter rule."""
    key_case = next(c for c in cases if c['kind'] == 'rsa_encrypt')
    public_blob = bytes.fromhex(key_case['public_blob'])
    private_blob = bytes.fromhex(key_case['private_blob'])
    block = bytes.fromhex(next(c['input'] for c in cases
                              if c['kind'] == 'oaep_decode' and c['case'] == 'k128_no_01_delimiter'))
    n = int.from_bytes(public_blob[4:], 'little')
    m = int.from_bytes(block, 'little')
    assert 0 < m < n
    ciphertext = pow(m, 65537, n).to_bytes(128, 'little')
    assert any(ciphertext)
    start = len(emu.calls)
    public, private = make_rsa_base(emu), make_rsa_base(emu)
    for obj, address, blob in [(public, 0x1400668E0, public_blob),
                               (private, 0x140066B50, private_blob)]:
        assert emu.call(address, obj, emu.alloc(len(blob), blob), len(blob), 1024) & 255 == 1
    setup_calls = emu.calls[start:]
    src, dst = emu.alloc(128, ciphertext), emu.alloc(128, b'\xA5' * 128)
    size = emu.alloc(4, u32(128))
    start = len(emu.calls)
    result = emu.call(0x140066D50, private, src, 128, dst, size, count=1_000_000_000) & 255
    error = emu.word(private + 8)
    assert result == 0 and error == 17, (result, error)
    assert emu.read(dst, 128) == b'\xA5' * 128
    return record(emu, start, 'rsa_decrypt', 'bad_oaep', '0x140066d50',
                  input=ciphertext.hex(), output='', result=result, error=error,
                  preimage=block.hex(), output_length=emu.word(size),
                  public_blob=public_blob.hex(), private_blob=private_blob.hex(),
                  object_changes={'0x8': error}, setup_calls=setup_calls)


def dh_vectors(emu):
    """Supply generated factors and random bytes at the original ABI boundary."""
    def prime(bits):
        while True:
            n = secrets.randbits(bits) | (1 << (bits - 1)) | 1
            d, s = n - 1, 0
            while d % 2 == 0:
                d //= 2
                s += 1
            for _ in range(40):
                x = pow(secrets.randbelow(n - 3) + 2, d, n)
                if x in (1, n - 1):
                    continue
                for _ in range(s - 1):
                    x = pow(x, 2, n)
                    if x == n - 1:
                        break
                else:
                    break
            else:
                return n

    def integer(n):
        return n.to_bytes(max(1, (n.bit_length() + 7) // 8), 'little')

    def factors(g, p, peer):
        return b''.join(u32(len(v)) + v for v in map(integer, (g, p, peer)))

    p = prime(256)
    g = secrets.randbelow(p - 3) + 2
    peer_exponent = secrets.randbits(64) | (1 << 63) | 1
    peer = pow(g, peer_exponent, p)
    short_p = prime(16)
    short_g = secrets.randbelow(short_p - 3) + 2
    cases = []
    for name, g_value, p_value, peer_value in [
            ('normal', g, p, peer),
            ('short_shared', short_g, short_p, short_p - 1),
            ('exponent_64_bits', g, p, peer),
            ('p_equal_g', p, p, peer),
            ('p_less_g', p + 1, p, peer)]:
        blob = factors(g_value, p_value, peer_value)
        obj = emu.alloc(2200)
        start = len(emu.calls)
        emu.call(0x1400670A0, obj)
        if emu.call(0x140067130, obj, 128) & 255 != 1:
            raise RuntimeError('DH Setup(128) failed')
        src = emu.alloc(len(blob), blob)
        result = emu.call(0x140067850, obj, src, len(blob)) & 255
        valid = p_value > g_value
        if bool(result) != valid:
            raise RuntimeError(f'DH factors unexpected result: {name}: {result}')
        if not result:
            cases.append(record(emu, start, 'dh_reject', name, '0x140067850',
                                input=blob.hex(), result=result))
            continue
        stream = bytearray(generated('dh', name, 'seed', 8))
        if name == 'exponent_64_bits':
            stream[0] |= 1
            stream[-1] |= 0x80
        else:
            # Check that the original forces both bits, not only the Rust code.
            stream[0] &= 0xFE
            stream[-1] &= 0x7F
        emu.set_random(stream)
        dst, size = emu.alloc(32, b'\xA5' * 32), emu.alloc(4, u32(32))
        result = emu.call(0x140067A10, obj, dst, size) & 255
        if result != 1 or emu.prng_used != 8:
            raise RuntimeError(f'DH getValue failed: {name}: {result}')
        reply = checked_output(emu, dst, 32, emu.word(size), blob)
        exponent = emu.read(obj + 1072, 8)
        expected_exponent = int.from_bytes(stream, 'little') | (1 << 63) | 1
        if exponent != expected_exponent.to_bytes(8, 'little') or emu.word(obj + 1068) != 64:
            raise RuntimeError('Original DH exponent does not have the expected 64-bit form')
        key_dst, key_size = emu.alloc(16, b'\xA5' * 16), emu.alloc(4, u32(16))
        key_result = emu.call(0x140067660, obj, key_dst, key_size) & 255
        if key_result != 1 or emu.word(key_size) != 16:
            raise RuntimeError(f'DH GetKey failed: {name}: {key_result}')
        key = checked_output(emu, key_dst, 16, 16, blob)
        if name == 'normal':
            shared = integer(pow(int.from_bytes(reply, 'little'), peer_exponent, p_value))
            if key != (shared * 16)[:16]:
                raise RuntimeError('DH peer exchange does not agree')
        if name == 'short_shared' and key != (integer(short_p - 1) * 8):
            raise RuntimeError('DH short key does not repeat cyclically')
        cases.append(record(emu, start, 'dh', name, '0x140067a10',
                            input=blob.hex(), seed=bytes(stream).hex(), output=reply.hex(),
                            session_key=key.hex(), exponent=exponent.hex(),
                            exponent_bits=emu.word(obj + 1068), result=result,
                            key_result=key_result, random_used=emu.prng_used))
    return cases


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--through", type=int, choices=range(1, 7), default=1)
    parser.add_argument("--reuse-rsa", action="store_true",
                        help="Reuse saved, sentinel-verified generated RSA vectors")
    args = parser.parse_args()
    emu = Emulator()
    vectors = dict(schema=1, build="0.0.381.572", pe_timestamp=hex(TIMESTAMP),
                   key_origin="generated test material only; built-in range was never read",
                   cases=[])
    vectors["cases"].extend(hash_vectors(emu))
    if args.through >= 2:
        vectors["cases"].extend(mgf_vectors(emu))
    if args.through >= 3:
        vectors["cases"].extend(oaep_vectors(emu))
    if args.through >= 4:
        if args.reuse_rsa:
            saved = [c for c in PREVIOUS['cases'] if c['kind'] in ('rsa_encrypt', 'rsa_decrypt') and c['case'] != 'bad_oaep']
            expected = {('rsa_encrypt', 'short'), ('rsa_encrypt', 'multi_chunk'),
                        ('rsa_decrypt', 'short'), ('rsa_decrypt', 'multi_chunk'),
                        ('rsa_decrypt', 'incomplete')}
            if len(saved) != 5 or {(c['kind'], c['case']) for c in saved} != expected:
                raise RuntimeError('No complete saved RSA vector set to reuse')
            for case in saved:
                if not any(c['address'] == case['source'] and c['sentinel']
                           and c['instructions'] > 1 for c in case['calls']):
                    raise RuntimeError('Saved RSA case lacks sentinel evidence')
                if case['result'] and case['output'] == case['input']:
                    raise RuntimeError('Saved RSA output copies its input')
            vectors['cases'].extend(saved)
            vectors['reused_rsa_stub_counts'] = PREVIOUS.get('reused_rsa_stub_counts', PREVIOUS.get('stub_counts', {}))
        else:
            vectors["cases"].extend(rsa_vectors(emu))
        vectors['cases'].append(rsa_bad_oaep_vector(emu, vectors['cases']))
    if args.through >= 5:
        saved_dh = [c for c in PREVIOUS['cases'] if c['kind'] in ('dh', 'dh_reject')]
        vectors['cases'].extend(saved_dh if args.reuse_rsa and len(saved_dh) == 5 else dh_vectors(emu))
    if args.through >= 6:
        from session_381 import session_vectors
        vectors['cases'].extend(session_vectors(emu, generated, record, checked_output))
        vectors['session_startup'] = {'size_limit': 4000, 'size_write': '0x140021087',
            'aes_branch': 'original non-AES-NI', 'pointer_writer': '0x140066db0',
            'setup': '0x14006d540', 'encrypt': '0x14006d590', 'decrypt': '0x14006d6a0'}
    vectors["stub_counts"] = emu.stub_counts
    VECTORS.parent.mkdir(parents=True, exist_ok=True)
    VECTORS.write_text(json.dumps(vectors, indent=2) + "\n")
    for case in vectors["cases"]:
        print(f"{case['kind']}: {case['case']}: original returned through sentinel")
    print(f"Wrote {len(vectors['cases'])} cases to {VECTORS.relative_to(VECTORS.parents[5])}")


if __name__ == "__main__":
    main()
