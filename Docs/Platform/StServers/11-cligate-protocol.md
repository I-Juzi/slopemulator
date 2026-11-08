# 11 — CliGate client protocol (original StServers)

Source: `CliGate.exe` (NP 1.20.3350, x64, no PDB), IDA 9.5 decompile
`BNSX-files/decomp/CliGate.c`. Addresses are image addresses (base 0x140000000).
Facts not checked against a live run are marked ❓.

## 1. Wire format

| Item | Format | Source |
| --- | --- | --- |
| Request line | `POST /<Protocol>/<Command> STS/1.0\r\n` (`HTTP/1.1` when the conn is HTTP) | sub_1400F1640 |
| Reply line | `STS/1.0 <status> <reason>\r\n` | sub_1400F2A80 |
| Default reasons | <200 `Informational`, <300 `OK`, <400 `Redirection`, <500 `Bad Request`, else `Server Error` | sub_1400F2A80 |
| Error reply | status 400, reason = error name (table below), body `<Error .../>` | sub_14012B940 |
| Sequence header `s` | `%u` + `R` (final) or `r` (partial); same as Rust `Transaction` | sub_1400F2790 |
| Error body | `<Error code= server= module= line= text=/>`; zero fields are left out | sub_140126380 |
| Error `server` | when 0, filled with the local server id (u16) | sub_14012B940 → sub_14012EC10 |

Error struct used by every handler: `{code u32, server u32, module u32, line u32, text*}`.
CliGate handlers use `module=1` and `line` = a source line number (for example 1293, 1336).

### Error-code names

`errcodes.tsv` (same folder) holds all 1550 `code → name` pairs, read from the
tables at 0x14027F170 (120 entries) and 0x14027F900. Common codes:

| Code | Name | Used by CliGate for |
| --- | --- | --- |
| 0 | Success | |
| 4 | ErrBadData | bad backend reply |
| 10 | ErrBadParam | missing body / field |
| 42 | ErrTimeout | backend timeout (logs the reply name) |
| 49 | ErrPermission | command not allowed in this state |
| 3030 | ErrBadConfig | cannot bind client app info |

## 2. Auth command enum

The table at 0x1401A35F0 (110 pairs `{name*, id}`) maps the `Auth` command name to an id
(sub_1400E4780). The ids are used by CliGate, AuthSrv and the route layer.

| Id | Command | Id | Command | Id | Command |
| --- | --- | --- | --- | --- | --- |
| 18 | GetMyUserInfo | 21 | KeyData | 22 | LoginStart |
| 23 | LoginFinish | 30 | RequestGameToken | 32 | RequestToken |
| 35 | VerifyMobiOtp | 36 | VerifySecondPwd | 37 | GetSecurityCardCodeIndex |
| 38 | VerifySecondaryAuth | 46 | CreateAccount | 58 | TokenKeyData |
| 96 | LoginTokenStart | 97 | LogoutMyClient | 98 | RequestCryptKey |
| 99 | WaitForAllowedLogin | | | | |

The full list (1 AccessTest … 110 Flush) is in the IDA table; the port copies it as one Rust enum.

## 3. CliConn dispatcher (CliGate.c line ~144380)

```
request ──► bind client app info (fail → 3030)
        ──► Auth/LoginStart or LoginTokenStart: reset session first (sub_1400D4390)
        ──► GameAccount cmd 3 ───────────► sub_1400D0500
        ──► Auth switch (below)
        ──► SecondaryAuth 19, IpAuthz, Profile 7, Mail 1, Gate/LookupGeolocation,
            Slot 4, Purchase, PurchaseLimit 23, VirtualCurrency 3, Auction …
        ──► otherwise: forward to the route grid (sub_14011A3A0), reply via CCliConn::ClientReply
```

| Auth id | Handler | Reply handler | Notes |
| --- | --- | --- | --- |
| 18, 20 | sub_1400CFA50 | | only before login (conn+300 = 0) |
| 21 KeyData | sub_1400D0130 | KeyDataReply sub_1400D2E60 | needs session (conn+328); `ErrPermission` after login |
| 22 LoginStart | sub_1400D09E0 | LoginStartReply sub_1400D4030 | LoginName max 64 chars |
| 23 LoginFinish | sub_1400D07B0 | LoginFinishReply sub_1400D3B20 → LoginFinishGateReply sub_1400D35C0 | needs flag conn+304 bit 2 (set by KeyData Complete) |
| 30 RequestGameToken | sub_1400D1940 | | |
| 32 RequestToken | sub_1400D1B10 | | |
| 35, 36, 38 | sub_1400D2380 | | secondary auth |
| 37 | sub_1400CFBA0 | | |
| 46 CreateAccount | sub_1400CF3E0 | CreateAccountReply sub_1400CF120 | needs salt + verifier |
| 58 TokenKeyData | sub_1400D1ED0 | TokenKeyDataReply sub_1400D55A0 | |
| 96 LoginTokenStart | sub_1400D0B60 | (local reply) | |
| 97 LogoutMyClient | sub_1400D1150 | LogoutReply sub_1400D4600 | |
| 99 WaitForAllowedLogin | sub_1400D27A0 | | |

All forwarded calls use sub_1400CEF30 with a 25 000 ms timeout (0x61A8).

## 4. Login sequence

```
Client                 CliGate                         AuthSrv
  │ Auth/LoginStart       │                               │
  │ <LoginName>           │ Auth/LoginStart               │
  │──────────────────────►│ <LoginName><NetAddress>──────►│
  │                       │◄──── <Session><KeyData>…      │ (userref, session saved conn+328)
  │◄── <Reply><KeyData>   │                               │
  │ Auth/KeyData          │ Auth/KeyData (t: header =     │
  │──────────────────────►│ userref) <Session><KeyData>──►│
  │                       │◄── <KeyData>[<Complete/>      │
  │                       │     <SessionKey>base64]       │
  │◄── <Reply><KeyData>   │ (sent in clear, then RC4 on)  │
  │ Auth/LoginFinish      │ Auth/LoginFinish + Session ──►│
  │──────────────────────►│◄── Licenses, Roles, PcCafe…   │
  │                       │ CCliConnTxn → gate reply      │
  │◄── reply              │ conn+300 = login id           │
```

| Step | Facts | Source |
| --- | --- | --- |
| LoginStart failure | 3 failures (conn+296) close the connection | sub_1400D2E60 |
| KeyData `Complete` | needs `SessionKey`; CliGate Base64-decodes it, builds the crypt filter (sub_140107980), sends the reply, then installs the filter (vtable+88) | sub_1400D2E60 |
| Log lines | `LOGIN`, `LOGIN-FAILED`, `LOGOUT`, `DISCONNECT` with userref, address, login name | sub_1400D2E60, sub_1400D4390 |
| Reset | LoginStart / LoginTokenStart clear session, userref, random buffers | sub_1400D4390 |

## 5. LoginTokenStart (id 96) — local reply

| Step | Detail |
| --- | --- |
| Input | `<ClientRand>` Base64, must decode to 32 bytes, else ErrBadParam line 1456 |
| ServerRand | 32 bytes: word 0 = sub_1400D9CE0(time) ❓, bytes 4..31 random, then XOR with ClientRand bytes 4..31 (sub_140108330) |
| Keys | needs the loaded server certificate (off_1402471B0: public key at +32/+44, signature at +56/+68); missing → code 4 line 1466 |
| Reply | `<Reply><ServerRand/><ServerPublicKey/><ServerSignature/></Reply>` |

## 6. Crypt filters

| Function | Builds | Detail |
| --- | --- | --- |
| sub_140107980(key, len) | CCryptFilter | two `Crypt::CKeyRc4` (send, recv) with the same key; plain RC4 KSA, no drop-N (vtable+16 is a no-op) |
| sub_1401078F0(k1, k2, len) | CCryptFilter | RC4 with separate send and receive keys |
| sub_1401077D0(ctx, is_server, cfg) | CTlsFilter | OpenSSL TLS filter; log `SocketCrypt [...] failed tls filter initialize` |

The KeyData session key feeds sub_140107980, so the client link is RC4 with one shared key, as in `rc4.rs`.

## 7. Connection identity (`Sts/Connect`, CliGate.c ~line 204990)

Every STS socket first says who it is. Two forms exist.

| Form | Fields |
| --- | --- |
| Header on `POST /Sts/Connect`: `X-Sts-Connect: <ConnType>; product=; index=; address=; deployment=; epoch=; notify=; version=; process=; program=; build=` | all numeric except `address` |
| Body `<Connect>` | `ConnType`, `Process`, `ProductType`, `AppIndex`, `Deployment`, `NotifyFlags`, `VersionFlags`, `Address`, `Program`, `Build`, `Epoch` |
| No `X-Sts-Connect` header | game client: ConnType 300, or 301 with `Authorization`; 302 for `Upgrade: websocket` |

Required: ConnType, Process, Program, Build (Program −1 → own program id). A bad `address` rejects the connection.

## 8. Open items

| Item | Next action |
| --- | --- |
| Certificate loader sub_1400D6030 | decode file format |
| LoginFinishGateReply body | decode the full reply XML |

