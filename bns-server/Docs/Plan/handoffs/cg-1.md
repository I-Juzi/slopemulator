# Handoff cg-1

## Done
- 11-cligate-protocol.md §8: open items closed. Certificate loader is out of scope (only LoginTokenStart uses it).
- §9 Forwarding: CCliConnTxn, IP check gate states, `t` = `<dc>$<UserId>`, reply path, errors.
- §10 LoginFinish chain (Auth/LoginFinish → Access/GetAppUserRights → Presence/Login), full client reply body.
- §11 RequestToken/RequestGameToken, generic gate path checks, ClientReply (partial replies).

## Not done
- §12 Session lifecycle is not in the spec yet. Facts that I verified in IDA but did not write:
  - Reset sub_1400D4390: log `%s, %s, %S, %s` (LOGOUT if conn+300≠0, else DISCONNECT; UserId, address, login name).
    Clears conn+300, UserId, names, flag conn+304 bit 2. When logged in: gate user logout, new client id.
  - Client Auth/LogoutMyClient (97) sub_1400D1150: logged in → reset, reply OK (line 1733), then close socket (vtable+72 ❓).
    Not logged in → ErrUserNotOnline 3007 line 1736.
  - Bus request `Cli/Logout` (<ClientId>, <UserId>) sub_1400D6A00 → sub_1400CEB30: no match → ErrClientNotFound 3018.
    `<Force>` → reset and close. Else CliGate sends the request to the game client (t = user), and on the client
    reply (LogoutReply sub_1400D4600) replies to the requester, resets, closes. Presence cmd 14 does the same.
  - Client idle timeout: `[IdleTimeout]` in config (sub_1400D6030), seconds ×1000; default and minimum 300000 ms
    (sub_1400D5FB0). Log `The idle timeout of ClientManager(%s) : %u seconds`.
  - AuthSrv lost: no special code found; pending txns end with ErrTimeout 42 after 25000 ms ❓.
- Rust card `Docs/Plan/cards/cg-1-rust.md` is not written.

## ❓
- IpCheck reply handler (state 2/3/4) not decoded. Rust phase 1 skips the IP check.
- Exact `f` header format with name suffix, Presence/Login field values, conn+600 = Licenses.
- Which service owns `Access/GetAppUserRights`.

## Blocked
- Bash is denied in this session, so I could not run tools (slice_runner, rust_api) or read StServer.ini.
