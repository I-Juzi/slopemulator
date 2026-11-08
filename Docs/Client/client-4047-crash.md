# Client error 4047 — root cause (build 381.8884, laptop)

## Summary
Error **4047** is NOT GameGuard, NOT the launcher, NOT network. It is a
reproducible **crash** (`c0000005` access violation) in **pathengine.dll**.
The client's crash handler shows the generic string
`ErrID_4047 = 실행중 오류가 발생하여 게임 클라이언트를 종료합니다.(4047)`
("A runtime error occurred; the game client terminates. (4047)"). The whole
`ErrID_4040`–`ErrID_4049` range shares that one text, so the number alone means
nothing — 4047 is just the crash-handler slot for this fault.

## Evidence (laptop, 2026-11-08)
- Windows Application log / WER APPCRASH, 3 identical launches:
  - `Faulting application: Client.exe 0.0.381.8884`
  - `Faulting module: pathengine.dll`
  - `Exception code: 0xc0000005`  `Fault offset: 0x000079d0`  (identical every time)
- During launch the client wrote ONLY GameGuard `.erl` logs, no game log.
- **No plugin logged** (keypatch logs to `C:\Users\Public\keypatch.log`, stays
  empty) → the winmm/version proxy loader is NOT active → our mods are inert.
  So the crash is the BASE client, mods not loaded.
- **Zero TCP packets** to the VPS during launch→crash (tcpdump) → pre-network.
- Client runs ~75 s (mostly Themida unpack) then crashes.

## Fault site (IDA, pathengine.dll, imagebase 0x10000000)
```
sub_100079D0 (RVA 0x79d0):     ; "lock this object" helper, this=eax
    mov   ecx, [eax+18h]       ; <-- AV: eax is null
    push  ecx                  ; lpCriticalSection
    call  EnterCriticalSection
    retn
```
Caller chain (file `.\cShape.cpp`, assert string `"_heldNode.valid()"`):
```
sub_1000CCE0: return dword_100BE624          ; global pathengine manager singleton
-> passes it as eax to sub_100079D0 -> locks [eax+0x18] -> null deref
```
`dword_100BE624` is written ONLY inside `DLLExport_GetIPathEngine`
(setter sub_1000CC60), and cleared to null by shutdown (sub_1000CCA0).
`sub_1000CC60` uses a plain malloc wrapper (sub_10001000) that only fails on真OOM,
so the null path is effectively never the allocator. Therefore `dword_100BE624`
is null because **a cShape is used before GetIPathEngine init, or after pathengine
shutdown nulled it** — i.e. likely a teardown/ordering crash that may be MASKING
an earlier fatal error (~75 s mark).

## Open question
Need the client-side call stack to tell primary-fault vs teardown-artifact.
WER keeps only transient Temp dumps here (cleaned). Enabled HKLM LocalDumps for
Client.exe → `C:\Users\Public\CrashDumps`, DumpType=2 (full). Next launch will
write a full dump to analyse the stack.

## Config oddities (may matter)
- `bin\local.ini`: Publisher=tencent, Language=chineses, GameGuard=NCW
- `bin\BnsCrashReport.ini`: Publisher=ncsoft  ← mismatch (frankenclient)
- start.bat args: `/sesskey /launchbylauncher /LoginMode 2 /ProxyIP:<server-ip>`
- pathengine.dll: 802816 bytes, PE ts 0x5178c03c, PathEngine SDK
  (exports DLLExport_GetIPathEngine). File on laptop pulled to
  <scratch>/pathengine.dll and ida/pathengine.dll.
