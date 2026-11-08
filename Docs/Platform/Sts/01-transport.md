# 01 — STS transport and framing

## Evidence and scope

Source S: `ida/delegate/sts/StsMsgSrv64.dll`, with its adjacent PDB. Addresses are virtual addresses at image base `0x180000000`.
❓ The DLL file version and its exact match to client build 381 are not established. The Connect builder writes protocol build 4000. This is not evidence that the client build is 4000.

This DLL is a message library for a game server that connects to the NP platform. It builds outbound requests and parses platform replies. It is not the NP authentication service. Its class contains an input byte buffer and an `IInetMsg` parser, not a listening socket. Sources: PDB type `StsMsgLib::StsMsgSrv`, `Initialize` at `0x180019020`, `WriteReceiveData` at `0x180017E80`, `Build` at `0x180016D70`, `Parse` at `0x18001ACF0`.

| Item | Checked result | Source S |
| --- | --- | --- |
| Request line | `POST /<protocol>/<command> STS/1.0\r\n` with flags 0 | `CInetMsgOut::Command`, `0x18001BE00`; URL writer `0x18001D1F0` |
| Response line | Version, decimal status, reason phrase, CRLF | `CInetMsg::ParseStartLine`, `0x18001DCC0` |
| Header line | Short name, colon, value, CRLF. STS writer adds no space after the colon. | `StartHeader`, `0x18001C160`; `EndHeader`, `0x18001B950` |
| Header end | Empty CRLF line | `Complete`, `0x18001BF90` |
| Body length | Header `l`, decimal body byte count, not character count | `Complete`, `0x18001BF90`; `ParseBody`, `0x18001E440` |
| Body | XML for the checked nonempty message builders | `0x1800010F0`, `0x180001400`, `0x180001800` |
| Empty body | Ping has zero body bytes | `0x1800012F0` |
| Stream parsing | Headers and body can span input chunks. `bytesUsed` reports consumed bytes. | `0x1800202E0`, `0x18001E440` |
| Header limit | Incomplete header accumulation rejects a size at or above `0x8000` | `0x1800202E0` |
| HTTP alternative | Flag `0x20` writes HTTP/1.1 and long headers. Flag `0x40` supports an HTTP chunk writer. These flags are zero in the checked STS builders. | `0x18001BE00`, `0x18001C160`, `0x18001BF90` |

The guide LobbyDaemon NPChannel uses a connected Session socket (`SharedChannel::Send`, `0x140004BA0` in the guide). Its NP send path copies STS bytes into internal Packet buffers (`0x140003AC0`). ❓ The internal 2-byte Packet header is not proven to be a wire prefix. Do not add it to the STS codec. The native transport still needs a check at Session/Connection send.

❓ TCP socket setup, TLS use, compression rules, and client connection type are not established by these builders. Use a stream codec, not an HTTP web-server parser. Port 6600 is task context only; no default port was recovered from this DLL.

## Headers and transactions

| Short header | Meaning | Evidence |
| --- | --- | --- |
| `l` | Content length | `0x18001BF90`, header lookup `0x18001E870` |
| `s` | Transaction metadata | `SetTransaction`, `0x18001C360` |
| `t` | Destination / routed name | Consume-token builder `0x180001400`, header lookup `0x18001E870` |
| `f` | Source / routed name | Header lookup `0x18001E870` |
| `c`, `e`, `x` | Content type, content encoding, transfer encoding | Header lookup `0x18001E870` |
| `m`, `p`, `r`, `v`, `z`, `a` | Stream, reply-to, refer-to, via, timestamp, accept | Header lookup `0x18001E870` |

`SetTransaction` writes a decimal nonzero transaction ID into `s`. A reply appends `R` when complete or `r` when incomplete. A nonzero sequence follows the letter. A zero ID removes the header. `WriteStsMsg` uses complete replies and sequence zero. Sources: `0x18001C360`, `0x180001020`.

The URL helper can also write `#<transaction>[r|R][sequence]`. The checked `Command` caller passes transaction zero to that helper. Thus do not add a fragment to normal requests. Sources: `0x18001D1F0`, `0x18001BE00`.

Generated framing example, not a captured packet:

```text
POST /Sts/Ping STS/1.0\r\n
l:0\r\n
\r\n
```

The body starts after the final empty line. No extra terminator follows the counted body.

## Configuration names only

Only option names were read from XML. No values were read or copied.

| File | Option | Proposed type |
| --- | --- | --- |
| `BnS_2020/_others/config/LobbyDaemon/bin/lobbyd.config.xml` | `np-address` | ❓ host string |
| same | `np-port` | ❓ port integer |
| same | `np-ping-interval` | ❓ duration; unit unchecked |
| same | `max-np-session-count` | ❓ count integer |
| same | `enable-np-recv-completion-log` | ❓ boolean |
| `BnS_2020/ManagementDaemon/bin/bns.common.config.xml` | `np-program-id` | signed integer in the DLL API |
| same | `np-game-code` | string, 1..8 bytes in the DLL API |
| same | `np-app-id`, `np-client-app-id` | ❓ GUID identifiers |

`Initialize` requires program ID >= 0 and a game-code string of length 1..8 (`0x180019020`). XML type bindings remain unchecked. No config sample is a source of local defaults.

## Local codec subset

❓ Local policy: support STS/1.0, CRLF, short headers, a required single `l`, and an opaque byte body. Keep unknown headers. Reject duplicate headers, folded headers, transfer encoding, malformed decimal lengths, and oversized frames. These strict rules are not claims about all original parser behavior. Do not implement HTTP chunking or cryptography in the first crate.

Implemented in `Program/Platform/Sts`. Header names use ASCII letters, digits, and hyphens. Header values and response reasons use printable ASCII. Request protocol/command segments use ASCII letters, digits, hyphens, underscores, and dots. URL fragments and percent escapes are outside this subset. Header names decode to lowercase. The codec rejects the long `Content-Length` alias to prevent ambiguous framing. Decimal lengths fit u32. Bodies remain opaque bytes. The codec does not parse XML.

❓ Local default limits: 32,767 header bytes, including the final delimiter; 1,048,576 body bytes. Both limits are explicit `Codec` fields. `decode` returns one frame plus its byte count, or incomplete without consuming input. The caller must close or reset the stream after an error. No listener or input-buffer owner exists in this crate.
