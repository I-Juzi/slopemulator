# Session crypt oracle — build 0.0.381.572

## Sources and start-up values

| Fact | Source in the original |
| --- | --- |
| The factory type is 3. The object size is 784 bytes. It creates two real AES objects. | `PacketCrypt::Create`, `0x14005EE80`; session vtable `0x140099B88`; Encrypt/Decrypt entries `0x140099BA8` and `0x140099BB0` |
| SetKey accepts the generated 16-byte key and selects slot 0 for send and receive. | `SessionPacketCrypt::SetKey`, `0x14005F1E0` |
| GenerateKey sets up the other AES slot and sets the pending byte. | `SessionPacketCrypt::GenerateKey`, `0x14005F240` |
| The size limit is 4000 bytes of padded ciphertext. It excludes the 2-byte header. | `Network::Initialize`, `0x140020E70`, write at `0x140021087`; Encrypt read at `0x14005F3F1` |
| The AES function pointers need start-up values. The harness uses the original non-AES-NI branch. | `CryptoKey::Initialize`, `0x140066DB0`, writes at `0x140066E05`, `0x140066E13`, `0x140066E46` |
| The three original table functions are setup, encrypt, and decrypt. | `0x14006D540`, `0x14006D590`, `0x14006D6A0` |

IDA `is_loaded` returned false for the size and AES pointer globals. Their displayed `0xFF` bytes are not file data. The harness sets these globals in emulator memory only. It does not call the network initializer.

## Coverage

The original runs 25 session cases. All 25 return through the sentinel.
Nine encrypt and nine decrypt cases have changed output bytes.
The seven rejection cases use the return code as output evidence. They do not count as successful cryptographic output.

The five fixed-key pairs cover 16 bytes, 32 bytes, 7 bytes, 17 bytes, and the 4000-byte limit.
The tests cover flag false and flag true.
The short inputs receive zero padding.
The four rotation pairs cover the last packet on the old key and the first packet on the new key in both directions.
The original uses the old send key once after GenerateKey. It then changes the send slot.
It keeps the old receive key until a packet has the new tag. It then clears the old AES key and changes the receive slot.
Both test objects use the real GenerateKey function. A controlled CRT rand stub supplies the same generated replacement key to each object.

The rejection calls return 3 for no key, 7 for an invalid tag or inactive slot, 8 for 4001 input bytes, 9 for a buffer of 18 bytes with 17 input bytes, and 10 for a truncated packet.

## Rust comparison boundary

`SessionCrypt` in `crypto.rs` has explicit slots. It does not contain packet headers or send/receive state.
The test adapter builds the 2-byte header. It selects the explicit slot from the saved original state.
All 18 successful cryptographic outputs match at this boundary.
This does not establish automatic key rotation in Rust.
The original no-key slot-0 state cannot be represented by the Rust constructor. Its Rust comparison is not run.
The invalid-tag and truncated-body cases reject on both sides. The original returns 7 and 10. Rust returns InvalidSessionCode(12) and IncompleteAesBlock.

Four tests stay ignored because the two interfaces differ.
The size-limit and small-buffer cases return successful ciphertext in Rust.
The inactive-slot case returns successful plaintext in Rust.
The automatic-change case uses the old slot in Rust unless the caller supplies the new slot.
`SESSION-DIFFERENCES.json` stores the exact generated inputs, keys, original outputs, and Rust outputs.
`IGNORED-RESULT.txt` records the deliberate run of all four ignored checks. All four fail as expected.
`SESSION-RESULT.txt` records the successful Rust output audit.

## Stubs and limits

The existing built-in RSA exclusion and access hooks are unchanged.
The binary and IDA database were not patched or saved by these tools.
The harness uses controlled TLS PRNG output and controlled CRT rand output.
It can also stub memory copy, move, set, compare, zero-filled allocation, no-op free/HeapFree, and fixed GetProcessHeap.
The vector file contains current and reused stub counts.
The hardware AES branch is not tested.
No BNSHelper, BloodCert, client executable, network, or gateway operation is part of these tests.

The separate original RSA zero-ciphertext case has no sentinel return.
Its trace ends at `0x14006E734` with a zero return address.
Its cause is ❓ unverified. It was not retried.
Rust rejects 128 zero ciphertext bytes with InvalidOaepDelimiter.
The active bad-OAEP comparison instead uses a nonzero generated ciphertext from the saved missing-delimiter OAEP vector.
The original returns 0 and sets error 17. Rust rejects it with InvalidOaepDelimiter.
