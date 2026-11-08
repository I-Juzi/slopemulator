# STS step 2 report

**Result:** The password path is partial. Real password authentication remains blocked. The default backend rejects login.

| Part | State | Evidence |
| --- | --- | --- |
| 1. Password protocol map | partial | `06-password-login.md`: command map `0x180028020`; dispatcher `0x180016D70` has no password handlers. PDB records establish API fields, not wire bodies. |
| 2. Program/game binding | partial | Connect carries Program at `0x1800010F0`. ConsumeGameToken carries GameCode at `0x180001400`. ❓ Client GameCode carrier remains unknown. |
| 3. State machine/accounts | partial | `Program/Platform/Sts/src/password.rs`: semantic records, pure states, verifier store and backend interface. Tests use generated accept/reject fixtures, not real password proofs. |
| 4. Token authority/adapter | done | Local single-use tokens check portal/program/game binding and expiry. Tests cover replay, concurrent consumption and revocation. Feature `lobbyd-adapter` implements `lobbyd::np::Np`. |
| 5. TCP listener | not started | No verified login body schema or working audited password backend. |
| 6. Tests and Clippy | done | Required workspace commands passed. STS feature checks also passed. |

**Tests:** Workspace: 264 passed, 0 failed, 6 ignored. The prior 244 tests still pass. STS with all features: 41 passed, 0 failed.
**Clippy:** Workspace/all-targets and STS/all-targets/all-features passed with `-D warnings`.
**Spec:** `bns-server/Docs/Platform/Sts/06-password-login.md`. SRP-family initialization: `0x180019020`; wrapper factory: `0x18002B8C0`; key-data interface: `0x18002B730`.

**Boundaries:**
- `srp` 0.6.0 states that it has no independent security audit. The trial implementation and its dependencies were removed. No real password backend ships.
- ❓ The vendor SRP variant, group size, hash, proof encoding and build-381 compatibility remain unknown. No vendor-secret dependency was established.
- No signature-path implementation, restricted client-binary read, licence-check work or anti-cheat work occurred. No licence/anti-cheat boundary address was encountered.
- The STS IDA lease is closed. No other database was opened or saved. No binary was patched.

**Open questions:** ❓ Client Connect/login/token bodies; platform error codes; client GameCode carrier; token-type selection; original session guards and deadlines.
**Next job:** Obtain approved server-side wire evidence. Confirm the standard and an audited backend. Implement real proof verification before a listener. Add attempt deadlines and rate limits.

Existing tests and `tools/emu` were not edited. LobbyDaemon source and default behavior did not change. Git checks were not approved. No commit was made.

---

# Prior step 1 report

**Result:** The protocol spec is partial. The Rust codec is complete for its stated subset. ❓ Build-381 login is not yet verified.

| Part | State | Evidence |
| --- | --- | --- |
| 1. Transport and messages | partial | `01-transport.md`: framing `0x18001BE00`, `0x18001BF90`. `02-messages.md`: consume-token fields `0x180001400`, `0x18000DD10`. Client login wire bodies remain open. |
| 2. Login flow | partial | `03-login-flow.md`: shared PDB message types and command map `0x180028020`. ❓ Client state order and signature trust remain open. |
| 3. LobbyDaemon token path | done | `04-token-validation.md`: guide-348 sender `0x1400D0220`, reply checks `0x1400D3500`; links to the local `Np` stub. |
| 4. Local server design | done | `05-local-server.md`: Argon2id accounts, TLS launcher API, single-use 60-second keys, token authority, and validation adapter. All proposed behavior has a ❓ scope marker. |
| 5. Rust crate | done | `Program/Platform/Sts`: bounded STS codec, transaction metadata, command names, and consume-token records. No listener or login handler. |
| 6. Checks | done | Workspace tests and strict Clippy passed. |

Specs: `bns-server/Docs/Platform/Sts/01-transport.md` through `05-local-server.md`.

**Tests:** 244 passed, 0 failed, 6 ignored. This includes 20 new generated STS tests. The prior 224 tests still pass.
**Clippy:** Passed with `--workspace --all-targets -- -D warnings`.
**Commands:** `cargo test --manifest-path bns-server/Cargo.toml`; `cargo clippy --manifest-path bns-server/Cargo.toml --workspace --all-targets -- -D warnings`.

**Boundaries:**
- Token-start response types include a server signature. ❓ Its algorithm, trust source, and need for a vendor secret remain unknown. That path stopped.
- No restricted client binary, licence-check code, anti-cheat code, or cryptographic implementation was inspected. No licence or anti-cheat boundary address was encountered.
- Both IDA database leases are closed. No explicit database save was requested. No binary was patched.

**Open questions:** ❓ STS DLL release identity; client Connect/login/token XML; native socket framing; app-ID binding; original token lifetime.

**Next job:** Resolve the client wire schema and signature trust from an approved source. Stop if acceptance needs an unavailable vendor secret. Then implement the local token authority and `Np` adapter.

Existing tests and `tools/emu` were not edited. Git diff checks did not run because they required approval. No commit was made.
