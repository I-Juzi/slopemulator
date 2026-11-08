# Build 0.0.381.572 — crypto comparison

| Item | Original address | Cases run | Match / mismatch / not run |
| --- | --- | --- | --- |
| 1: Digest | `0x14006F600` | 3 | Match: 3 |
| 2: MGF | `0x140069750` | 3 | Match: 3 |
| 3: OAEP | `0x1400699D0`, `0x140069D30` | 8 encode; 12 decode | Match: 20 |
| 4: RSA | `0x1400668E0`, `0x140066B50`, `0x140066A10`, `0x140066D50` | Generated key setup; 2 encrypt; 4 decrypt | Match: all |
| 4: Bad OAEP | `0x140066D50` → `0x140066580` | Nonzero generated ciphertext | Match: original return 0, error 17; Rust InvalidOaepDelimiter |
| 4: Zero ciphertext | Original result not claimed | Rust-only: 128 zero bytes | Rust InvalidOaepDelimiter; original not counted |
| 5: DH | `0x140067850`, `0x140067A10`, `0x140067660` | 5 | Match: 5 |
| 6: Session bytes | `0x14005F3B0`, `0x14005F4D0` | 9 encrypt; 9 decrypt | Match: 18 with explicit Rust slots and a test header adapter |
| 6: Session errors | Same | 7; codes 3, 7, 8, 9, 10 | Match: 2 rejections; mismatch: 3; Rust no-key comparison not run: 2 |
| 6: Key-change state | Same | Old/new key packets in both directions | Original state verified; Rust has no automatic state; mismatch |

The 36 saved cases remain. The vector file now has 62 cases with sentinel evidence.
All 18 successful session outputs differ from their inputs. Rejection tests compare error returns, not unchanged buffers.
Both flags, short-block zero padding, the 4000-byte limit, and both key slots were tested.

| Mismatch input | Original output | Rust output | Cause |
| --- | --- | --- | --- |
| `too_large`: byte `38` × 4001; capacity 4018 | Error 8; no ciphertext | Ok: 4016 ciphertext bytes | No packet size limit in SessionCrypt |
| `small_buffer`: bytes `00` through `10`; capacity 18 | Error 9; no ciphertext | Ok: 32 ciphertext bytes | Rust allocates output; no capacity argument |
| `wrong_active_slot`: tag 9; send=receive=0; both generated keys set | Error 7; no plaintext | Ok: `269ce958a1de85bd5c099eccdd8e276e` | Rust selects the explicit slot without an active-slot check |
| `rotate_send_new`: `381 rotation rotate_send_new`; flag true; previous slot 0 | 34-byte packet; header `08b0` | Without caller slot change: 34-byte packet; header `08a0`; different ciphertext | Rust has no pending/send/receive state |

Exact mismatch inputs, generated keys, and both outputs: `lobbygate-rs/tools/emu/SESSION-DIFFERENCES.json`.
The four mismatch tests stay `#[ignore]` with reasons. Their deliberate run failed all four checks, as expected; see `IGNORED-RESULT.txt` in the same directory.
**Start-up:** the harness sets limit 4000 from `0x140021087` and the original software AES pointers from `0x140066DB0`. These globals have no loaded file bytes. It uses the real factory, key setter, and GenerateKey.
**Stubs that can change results:** controlled TLS PRNG; controlled CRT rand; memcpy/memmove/memset/memcmp; zero-filled allocation; no-op free/HeapFree; fixed GetProcessHeap. Only the two random stubs, memcpy, and memset ran.
**Open:** original zero-ciphertext return and cause are ❓ unverified; the prior trace ended at `0x14006E734` with return address 0. Rust cannot represent unconfigured slot 0. The hardware AES branch is not tested. Four packet/state gaps remain.
`cargo test --manifest-path lobbygate-rs/Cargo.toml`: **PASS — 52 passed, 0 failed, 4 ignored**. Full output: `lobbygate-rs/tools/emu/CARGO-RESULT.txt`.
`--through 6 --reuse-rsa` and the evidence audit pass. Sources and test limits: `lobbygate-rs/tools/emu/SESSION-NOTES.md`.
`crypto.rs`, the spec, and the built-in RSA protection are unchanged. No binary or IDA patch/save command was used. IDA is closed.
