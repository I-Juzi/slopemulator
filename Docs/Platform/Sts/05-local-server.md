# 05 — Proposed local STS server

## Status and acceptance limit

**❓ All server behavior in this file is a proposed local design. It is not recovered vendor behavior.**
The design can support a local launcher and the Rust LobbyDaemon. It cannot yet promise login by an unchanged build-381 client. Client Connect fields, login wire bodies, and server-signature trust remain unresolved. See `03-login-flow.md`.

Do not replace client trust checks with success stubs. Do not open restricted client binaries to fill these gaps. Obtain an approved protocol document or generated trace from an authorized local test implementation. Stop any path that requires an unavailable vendor secret.

## Components and trust

| Component | Proposed responsibility | Trust rule |
| --- | --- | --- |
| Launcher HTTPS API | Verify account password. Return a single-use launcher session key. | Use locally controlled TLS trust. Require valid TLS. This API is not the STS protocol. |
| Account store | Stable portal GUID, distinct game-account GUID, display identity, roles, password hash | Store Argon2id PHC password hashes with a unique random salt. Never store plaintext passwords. |
| Session-key authority | Issue, expire, revoke, and consume launcher keys | Generate at least 256 random bits. Store a digest, not the bearer value. Expire at 60 seconds. |
| Client STS endpoint | Decode frames and run an explicit authentication state machine | No listener until the client wire and trust boundaries are resolved. No implicit success. |
| Token authority | Issue opaque NP game-token GUIDs and bind identities | Generate nonzero 128-bit random values. Keep server-side state. Do not treat a token as proof without a stored record. |
| LobbyDaemon adapter | Implement `Np` over the authority and explicit presence policy | Bind game code and app policy at adapter creation. Reject unknown callers and tokens. |
| Optional native NP endpoint | Serve original game-server requests with STS framing | Later work. Requires transport ownership and required follow-up messages. |

Keep a single token authority for issue and consume. Prefer an in-process interface for the first integration. If separate processes are required, use a local authenticated interface with access control. Do not expose validation to the public client endpoint.

## Launcher API contract

❓ Proposed local API; not vendor wire names:

| Operation | Input types | Output types | Errors |
| --- | --- | --- | --- |
| `POST /local/v1/login` over TLS | Login identifier string, password secret, requested app identifier | Opaque session-key secret string, integer expiry seconds, portal GUID | Generic bad credentials, rate limit, unavailable |
| Internal key consume | Key secret, app identifier, attempt context | Stable account reference | Absent, expired, wrong binding, consumed |

Set a short request/body limit. Limit authentication work per account and connection. Use one generic credential error. Do not return password hashes or secrets in errors. Do not log request bodies. Use a maintained TLS implementation. Do not reuse the launcher session key as the TLS key or an NP game token.

Consumption must be atomic. Check expiry and app binding before marking a key consumed. Reject at `now >= deadline`. Two concurrent valid requests must yield one success. Restart invalidates outstanding in-memory keys. Revoke related records when an account is disabled or a password changes.

Argon2id storage does not establish compatibility with a native SRP password login. If native login needs a verifier, specify a separate supported design after the protocol boundary is resolved. Do not derive or copy vendor verifier internals in this step.

## Token issue and validation

❓ Proposed token record:

| Field | Type / rule |
| --- | --- |
| Game token | Nonzero opaque 16-byte identifier; treat as a bearer secret |
| Portal ID | 16-byte GUID; must match the consume request destination |
| Game-account ID | Separate stable 16-byte GUID |
| Game code | Validated 1..8-byte configured string |
| Client app / server app | Typed local identifiers; exact wire binding remains open |
| Issue / expiry | Monotonic local deadline; explicit configured lifetime |
| Use state | Unused, consumed, or revoked |
| Identity | Nonempty local login/user names, roles, explicit slot/profile/grade data |

No default token lifetime was recovered. Require an explicit positive local setting. Do not reuse the 60-second launcher-key lifetime as a claimed original NP token lifetime.

Consume under one lock or transaction: find token; check expiry; check portal ID; check game/app policy; check account state; mark consumed; return immutable identity. A wrong binding must not consume another account's token. Expired tokens must fail even before an expiry sweep. Never return partial identity as success.

Keep presence ownership separate from token use. `Np::login_presence` can fail after token consumption. Define local retry policy explicitly. Do not undo token consumption automatically on later login failure. Keep the current deterministic stub for existing tests.

## Endpoint sequence and failure handling

❓ The launcher obtains a session key through the new HTTPS API. The client then needs a supported token-login exchange. Its relationship to `/SessKey` remains open. After authentication, a game-token request can create an NP token record. The client presents its NP game token and portal GUID to LobbyGate. LobbyDaemon consumes that pair through the new adapter.

Reject a token request before authentication. Reject an out-of-order login operation. Keep transaction IDs scoped to a connection. Bound pending requests. Cancel pending work on disconnect. Use monotonic time for deadlines. Do not infer a duration from an event ID.

No protocol response status is assigned to a guessed error yet. Keep typed internal errors until a verified wire mapping exists.

## First Rust milestone

| Included | Excluded |
| --- | --- |
| Dependency-free STS/1.0 frame types | Network listener and TLS setup |
| Explicit request and response start lines | Login handlers and acceptance claims |
| Decimal byte-length framing with CRLF | Password / SRP / signature implementation |
| Transaction request/reply metadata | XML schemas for unknown client messages |
| Checked command-name classification | Vendor account records or config values |
| Opaque binary bodies and unknown headers | Native LobbyDaemon transport adapter |
| Generated round-trip and rejection tests | Changes to `tools/emu` or existing tests |

The codec uses a strict local subset from `01-transport.md`. It does not claim to accept every frame accepted by the original parser. It must return incomplete without consuming a partial frame. It must decode one complete frame at a time and preserve trailing input.

## Required tests for later work

All fixtures must be generated.

| Area | Required cases |
| --- | --- |
| Codec | Empty body, binary body, UTF-8 byte length, all split positions, two coalesced frames, truncated body, bad lengths, duplicate headers, CRLF injection, limits |
| Launcher key | Success once; wrong app; exact expiry; race; revocation; restart |
| Game token | Correct pair; wrong portal/game/app; zero/unknown token; replay; exact expiry; concurrent consume |
| LobbyDaemon | Adapter through D6/D9; explicit rejection; valid identity; presence conflict; timeout; no automatic vendor access |
| Client acceptance | Approved evidence for Connect/login schemas and trusted local authentication. Stop at a vendor-secret boundary. |

## Next job

Resolve the build-381 token-login wire schema and signature trust from an approved source. Do not add a listener before this check. Then implement the local authority and an `Np` adapter. Keep the original-platform endpoint as a separate compatibility task.
