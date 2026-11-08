# 03 — Packet framing and field formats

Target build: LobbyGate `0.0.381.572` (2020). Reference build: `0.0.348.527` (2019, with PDB).
Status: **draft 2** — frame rules, format codes, repeat groups, fragments and the 3rd-party hook verified ✅ in 348 and 381.

All integers are **little-endian**.

## 1. Two link types

| Link | Port (config) | Buffer handler | Crypto |
| --- | --- | --- | --- |
| Game client ↔ LobbyGate | `client-port` (10900) | `ClientPacketBufferHandler` | handshake RSA, then AES-128 (spec 02) |
| LobbyDaemon ↔ LobbyGate | `lobby-daemon-addr:port` (21801), outgoing | `TrustedPacketBufferHandler` | **none** (plain) |

LobbyGate opens no listener on `internal-port`. It only advertises that port to LobbyD in D1 (05 §8.5).

Receive buffer per session: 4008 bytes (`SessionRecvBuffer`, `PacketBuffer::Buffer[4008]`). `Session::handlePacket` cuts as many whole frames as the buffer holds, hands each to the port's session handler, then moves the remainder to the buffer start. A handler that returns `false` closes the session on the client link. On the LobbyD link the queued path ignores `false` (05 §6.1).

## 2. Plain packet (inside every frame)

```
i16 size     total bytes of this packet, including `size` itself   (≥ 2)
i16 cmd      command number (dispatch tables: see 04-dispatch)
u8  body[size-4]
```

The `Packet` object keeps `CurHeader` → `size`, and `CurHeader[1]` → `cmd`.

## 3. Trusted frame (LobbyDaemon link)

The frame **is** the plain packet. Nothing else surrounds it.

| Check | Action | 348 | 381 |
| --- | --- | --- | --- |
| fewer than 2 bytes buffered | wait | ✅ | ✅ |
| `size < 2` or `size > 4000` | log `== CRASH == … TrustedPacketBufferHandler; invalid packet size` + alert; close | ✅ | ✅ (`(size-2) > 0xF9E`) |
| fewer than `size` bytes buffered | wait | ✅ | ✅ |

Send side: the packet goes out unchanged (`ToBuffer` returns `size`).

## 4. Client frame

```
u16 hdr
u8  payload[len]          len = 4 × (hdr & 0x0FFF)      if (hdr >> 14) != 0
                          len = hdr & 0x3FFF            if (hdr >> 14) == 0   (plain — rejected, see below)
frame size = len + 2
```

`hdr >> 12` is the crypt **code** (spec 02): `0x4` initial key (client→server handshake), `0x5` client handshake key (server→client), `0x8`–`0xB` session keys, `0xE` broadcast key (server → client only, 05 §3). Payload length is always a multiple of 4.

Receive steps (`ClientPacketBufferHandler::ToPacket`):

| Step | Rule | Result on failure |
| --- | --- | --- |
| 1 | `len > 0xFFFF` | close |
| 2 | fewer than `len + 2` bytes buffered | wait |
| 3a | **no crypt yet** and `hdr >> 14 == 0` | log `Handshake decrypt failed, code 'plain' not acceptable`; close |
| 3b | no crypt yet, crypt mode 2 | create the handshake crypt with algorithm = `hdr >> 14`, then decrypt (RSA) | close on error |
| 3c | crypt present, mode ≠ 5 | log `invalid module, mode:%d`; close |
| 3d | crypt present, code ∉ {8,9,10,11} | log `invalid enc-code`; close |
| 3e | crypt present | AES decrypt in place; if a 3rd-party security module is attached, it also decrypts `buffer+4 … size-2` | close on error |
| 4 | inner `size < 2` or `size >` decrypted length | log `invalid packet size`; close |
| 5 | packet = `buffer + 2`; consumed bytes = full frame | — |

381: the same log strings and the same checks are present in `0x1400330C0`. The exact compiled constants for step 3d are not re-read yet ❓.

Send side (`ToBuffer`): frame size = `4 × (hdr & 0xFFF) + 2`. The encrypt step writes `hdr` (spec 02 §5–6).

**Rule for Rust:** the client handshake and session crypto sit *between* the frame header and the plain packet. The plain packet (§2) starts at frame offset 2, after decryption. Ciphertext padding (AES zero padding, RSA block size) is why `size` (inner) can be smaller than `len`.

## 5. Field format codes (`PacketBuffer::Compose` / `Decompose`)

Format strings such as `"hhb"` describe the body after `size`. The first `h` is `cmd`. 381 uses exactly the same 17 codes (`0x140033EE0`).

| Code | Wire bytes | Value | Decompose checks |
| --- | --- | --- | --- |
| `c`, `Q` | 1 | u8 / i8 | bounds |
| `h`, `u`, `E`, `U` | 2 | i16 / u16 (letters only tell types apart in C++) | bounds |
| `C` | 2 | repeat count (i16 written, u16 read); opens a repeat group, see §6 | bounds |
| `d`, `i` | 4 | i32 / u32 | bounds |
| `f` | 4 | f32 (C++ passes a double; it is narrowed to f32) | bounds |
| `q` | 8 | i64 / u64 | bounds |
| `o` | 16 | 128-bit id (`id128`, e.g. NP tokens and GUIDs) | bounds |
| `b`, `K` | 2 + n | `i16 n`, then n bytes | n ≥ 0 (`src size underflow`); n ≤ destination size (`not enough buffer`); bounds |
| `s` | 2 + 2n | `i16 n` = UTF-16 **code units**, then n × UTF-16LE, **no** terminator | n ≥ 0 (`length underflow`); destination ≥ n+1 (`string buffer is too small`); bounds (`length overflow`) |
| `P` | 5 + n | `u8 typeId` (> 0), `i16 count`, `i16 n`, n bytes — a typed "property" blob | typeId > 0 (`invalid property type id`); count ≥ 0; n ≥ 0; destination ≥ n |
| `;` | 0 | closes a repeat group (§6), no data | — |
| other | — | `Unknown type %C`; Compose crashes the process on purpose, Decompose returns failure | — |

Compose writes nothing past the 4000-byte limit: on overflow it logs `Compose overflow.` and crashes on purpose (`CrashHandler::MakeCrash`). Rust: return an error and close the session. The crash itself is not observable on the wire.

## 6. Repeat groups (`C` … `;`)

`C` opens a repeat group and the matching `;` closes it. On the wire, `C` is an `i16 count`. Then the group fields follow `count` times. The writer sets `count` to 0 first and increments it for each item, so the count is correct even when the items continue into later fragments (§7).

| Rule | Value | 348 | 381 |
| --- | --- | --- | --- |
| Open / close characters | `C` (0x43) / `;` (0x3B) | ✅ `PacketFormatIterator::skipRepeatFormat` | ✅ `0x14001D450` |
| Nesting | groups can nest; maximum depth **3** (`too many nested Repeat Format`) | ✅ | ✅ `0x14001D550` (`top+1 >= 3`) |
| Free space for the count | ≥ 2 bytes, else a new fragment starts first | ✅ | ✅ |
| Empty group (count = 0) | the reader skips to the matching `;` | ✅ | ✅ |
| Reader | reads `count` as **u16** (`RepeatPacketDecomposer::PopRepeatCount`) | ✅ | ✅ `0x14000AAD0` |

Examples from the 2019 code (fragment formats, see §7):

| Format | Meaning |
| --- | --- |
| `hiqhCqohsbqQQi;` | cmd 28 character list: `q now`, `h timezone bias`, repeat {`q o h s b q Q Q i`} |
| `hiCqiQQqqssissbbCiiibb;;` | a group with a nested group inside |
| `hiqCiiiii;CiiQiqq;iqiq` | two groups one after the other, then plain fields |

## 7. Fragment packets (messages larger than 4000 bytes)

A long message is split into a chain of normal plain packets (§2). Every fragment has the same `cmd` and an 8-byte header:

```
i16 size      bytes of this fragment (≤ 4000)
i16 cmd       the message command (same in every fragment)
i32 mark      see the table
u8  data[size-8]
```

| Fragment | `mark` |
| --- | --- |
| only one fragment | total data length (= `size - 8`) |
| first of several | total data length of the whole message (sum of every `size - 8`) |
| middle | `-1` (more data) |
| last of several | `0` (end) |

The format string of a fragment message starts with `hi` (`cmd`, `mark`). The writer starts writing at format index 2.

**Writer rules** (`FragmentPacket`, `FragmentPacketSplitter`, `Packet::ComposeV`):
- Fields are written one at a time. A field is **never** split. When the next field does not fit in the 4000-byte fragment, a new fragment starts.
- On the client link, **each fragment is its own client frame** and is encrypted on its own (`PacketBuffer::encrypt` walks the buffer list).
- Fragment messages that LobbyGate sends to the client in 348: cmd 28 (`SendGetCharacterInfoListOk`), `SendListGoodsOk`, `SendTakeGoodsOk`, `SendListLobbyServiceItemInfoOk`, `SendGetServiceEntryListOk`, and the contents-rule list (`ContentsRuleConfig::MakeListPacket`). Command numbers are in 04-dispatch.

**Reader rules** (`FragmentPacketConcatenater::Add`, used for LobbyDaemon fragment commands):

| `mark` | Condition | Result |
| --- | --- | --- |
| > 0, nothing buffered, `size - 8 == mark` | single fragment | complete; use the data in place |
| > 0, other cases | first fragment; store `cmd` and `mark` as total | wait for more |
| `-1` | buffer must not be empty (`Size = 0`) | append; wait |
| `0` | after append, buffered length must equal total (`Size != total`) | complete |
| < -1 | — | fail (`total < 0`) |
| any | `cmd` must equal the stored `cmd`; buffered + new data must not exceed total | else fail (`add failed`) |

Only `data` is stored; the 8-byte headers are removed. The joined data is then decoded with the format string after `hi`.

381: the same constants (`-1`, `+6`, `size - 8`, `0`/`-1` branches) are in `FragmentPacketSplitter::AddBuffer` `0x140002400`, `Finalize` `0x140002520` and `FragmentPacketConcatenater::Add` `0x140002810` ✅.

## 8. 3rd-party security hook (`Security3rdParty`)

An optional second cipher step after AES, for an outside security product. Codes `0xA`/`0xB` mean "AES + 3rd-party step".

| Build | Result |
| --- | --- |
| 348 | `Session::_security3rd` (Session `+0x120`) is set to `nullptr` in all constructors. No code sets another value. ✅ |
| 381 | the same field at `+0x120` is passed to `PacketBuffer::encrypt` (`0x140034A60`, 21 call sites). The only stores write 0 (Session reset `0x140018490`, destructor `0x140017FB0`). ✅ |
| Vendor strings | none in either exe |

**Rule for Rust:** the hook is never active. Send only codes `0x8`/`0x9`. A received `0xA`/`0xB` frame cannot occur with the real server; treat it like `0x8`/`0x9` with no extra step (that is what the C++ does when the hook is null).

## 9. Open items

1. 381 constants for client step 3d.
2. Receive-side command list of fragment messages (LobbyD fragment table `0x1400A43D0`) — 04-dispatch.
3. Reader edge cases, low risk because the LobbyD link is trusted: a positive `mark` during a chain (the buffered data stays, `PacketConcatenateBuffer::Add` `0x14001DD10`; ❓ result for the stored total), and `mark` 0 with no chain. The current Rust reader replaces the total and accepts an empty message.
