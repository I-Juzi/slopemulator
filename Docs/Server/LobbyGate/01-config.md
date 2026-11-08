# 01 — Configuration

Target build: LobbyGate `0.0.381.572` (2020). Reference build: `0.0.348.527` (2019, with PDB).
Status: **draft 1** — key tables verified; semantics marked ✅ (verified in code) or ❓ (open).

## 1. Sources

| Item | 2020 (381) address | 2019 (348) symbol |
| --- | --- | --- |
| LobbyGate key table (`XmlCfgDef[]`, 40-byte entries) | `0x1400A4FF0` | `LobbyGateConfig::_Def` `0x140092D40` |
| bns-common load table (`XmlCfgDef[]`) | `0x1400A47F0` | `BnsCommonConfigLoad::_Def` `0x140092780` |
| bns-common rule table (`XmlCfgDefLocal`, 64-byte) | `0x1400A48E0` | `BnsCommonConfig::_Def` `0x140092640` |
| contents-rule load table (`XmlCfgDef[]`) | `0x1400A4DC0` | `ContentsRuleConfigLoad::_Def` `0x140092850` |
| contents-rule rule table (56-byte entries) | `0x1400A4AB0` | `ContentsRuleConfig::_Def` `0x140092A60` |
| Loader | `LobbyGate::initLog` `0x1400135C0`, `BnsCommonConfig::LoadBnsCommonConfig` `0x140002AC0`, `ContentsRuleConfig::LoadContentsRuleConfig` `0x140007EB0` | same names |

Table entry layouts (from the 2019 PDB, confirmed by the 2020 data):

```
XmlCfgDef        (40 B): u64 offset | wchar* name | i32 type | i32 reqOption | void* default | wchar* desc
  group header   : offset = -1 (348: 64-bit, 381: 32-bit 0xFFFFFFFF), name = NULL, desc = group name
XmlCfgDefLocal   (64 B): u64 offset | wchar* name | i32 xmlType | i32 defType | void* min | void* max | bool reload | void* default | u64 refOffset
rule table 56 B        : u64 offset | wchar* name | i32 xmlType | void* min | void* max | bool reload | void* default
xmlType enum           : 1 INT, 2 INT64, 3 FLOAT, 4 BOOL, 5 IP, 6 STR, 7 DIR, 8 ANSI, 9 GROUP
```

## 2. File format

All three files share one shape. The Rust loader must accept exactly this:

```xml
<?xml version='1.0' encoding='utf-8'?>
<config module='MODULE-NAME'>
    <option name='KEY' value='VALUE'/>
</config>
```

| Rule | Status |
| --- | --- |
| Single or double quotes for attributes | ✅ both appear in shipped files |
| Comments (`<!-- -->`) anywhere, including Korean text, UTF-8 | ✅ shipped files |
| `module` attribute names the file kind | ✅ values below |
| Unknown keys in a file are ignored | ✅ the 348 binary reads files that contain the 381-only key `ranking-service-mode` |
| BOOL values `yes` / `no` | ✅ shipped files; ❓ other spellings (`true`, `1`, case) are parsed in `XmlReader_sv64.dll` — not yet checked |
| Whitespace and case rules for keys | ❓ `XmlReader_sv64.dll` |

| File | `module` | Loaded from |
| --- | --- | --- |
| `lobbygate.config.xml` | `lobbygate` | ❓ the folder given with the `-config-dir` option or the exe folder (`CONFIG_DIR_OPTION`, `CONFIG_FILENAME`) |
| `bns.common.config.xml` | `bns-common-config` | ❓ same folder |
| `ContentsRule.lobbygate.config.xml` | `lobbygate-contents-rule-config` | ❓ same folder |

## 3. Keys

The 2020 tables, generated from the binary. "348→381" is empty when type, required flag, default and limits are the same in both builds.

### lobbygate.config.xml (module `lobbygate`)

| # | Key | Type | Required | Default if missing | Struct offset | 348→381 |
|---|---|---|---|---|---|---|
| 1 | `root-path` | DIR | yes | — | `0x0` |  |
| 2 | `log-path` | DIR | yes | — | `0x8` |  |
| 3 | `config-path` | DIR | yes | — | `0x10` |  |
| 4 | `cert` | ANSI | yes | — | `0x18` |  |
| 5 | `worker-thread` | INT | no | — | `0x20` |  |
| 6 | `max-client` | INT | no | `200000` | `0x24` |  |
| 7 | `dump-type` | INT | no | `1` | `0x28` |  |
| 8 | `timer-interval` | INT | no | `1000` | `0x2c` |  |
| 9 | `timer-max-time` | INT | no | `20` | `0x30` |  |
| 10 | `init-client-session-count` | INT | no | — | `0x34` |  |
| 11 | `client-session-hash-count` | INT | no | `1` | `0x38` |  |
| 12 | `predefined-world-id` | INT | yes | `4001` | `0x3c` |  |
| 13 | `client-close-timeout` | INT | yes | — | `0x40` |  |
| 14 | `internal-port` | INT | yes | `20900` | `0x44` |  |
| 15 | `client-port` | INT | yes | `10900` | `0x48` |  |
| 16 | `lobby-daemon-addr` | IP | yes | `127.0.0.1` | `0x4c` |  |
| 17 | `lobby-daemon-port` | INT | yes | `21801` | `0x50` |  |
| 18 | `max-idle-time-enable` | BOOL | yes | `yes` | `0x54` |  |
| 19 | `disconnect-after-idle-time` | BOOL | no | — | `0x55` |  |
| 20 | `max-idle-time` | INT | no | `300` | `0x58` |  |
| 21 | `use-channel-delayed-release` | BOOL | no | — | `0x5c` |  |
| 22 | `alert-send-address` | STR | no | — | `0x60` |  |
| 23 | `cooldown-complete-delay` | INT | no | `180` | `0x68` |  |
| 24 | `allow-client-ip-address-in-logs` | BOOL | no | `yes` | `0x6c` |  |

"—" means no default pointer: the field stays zero (`0`, `no`, empty string).

### bns.common.config.xml (module `bns-common-config`)

| # | Key | XML type | Required | Min | Max | Default | Reloadable | 348→381 |
|---|---|---|---|---|---|---|---|---|
| 1 | `p-type` | STR | yes | — | — | `live` | no |  |
| 2 | `ranking-service-mode` | INT | yes | `1` | `11` | `1` | no | new in 381 |
| 3 | `np-program-id` | INT | no | `0` | `10000` | `303` | no |  |
| 4 | `np-game-code` | STR | no | — | — | `bnsprog` | no |  |
| 5 | `np-app-id` | GUID | no | — | — | `AFACDF37-7714-474A-BEAD-B289447B5A2B` | yes |  |
| 6 | `np-client-app-id` | GUID | no | — | — | `6E599F8E-415E-4A4C-9999-07E9BBFFD17A` | yes |  |

The built-in GUID defaults are NCSoft's `bnsprog` (program/dev) IDs. The Rust code must not hard-code them; it uses whatever the operator's file supplies and treats a missing value as "unset".

### ContentsRule.lobbygate.config.xml (module `lobbygate-contents-rule-config`)

| # | Key | XML type | Required | Min | Max | Default | Reloadable | 348→381 |
|---|---|---|---|---|---|---|---|---|
| 1 | `dummy-data` | INT | yes | `0` | `1234` | `123` | no |  |
| 2 | `use-time-based-play-schedule` | BOOL | yes | — | — | `no` | no |  |
| 3 | `enable-play-warrior` | BOOL | yes | — | — | `yes` | no |  |
| 4 | `enable-fish-system` | BOOL | yes | — | — | `no` | no |  |
| 5 | `enable-account-contents` | BOOL | yes | — | — | `no` | no |  |
| 6 | `account-contents-required-pc-level` | INT | yes | `0` | `1000` | `0` | no |  |
| 7 | `account-contents-required-pc-mastery-level` | INT | yes | `0` | `1000` | `0` | no |  |
| 8 | `account-contents-required-account-top-pc-level` | INT | yes | `0` | `1000` | `0` | no |  |
| 9 | `account-contents-required-account-top-pc-mastery-level` | INT | yes | `0` | `1000` | `0` | no |  |
| 10 | `account-exp-quota-per-day` | INT | yes | `0` | `2147483647` | `0` | no |  |
| 11 | `account-exp-quota-reset-time` | INT | yes | `0` | `23` | `6` | no |  |
| 12 | `account-level-max` | INT | yes | `0` | `1000` | `50` | no |  |
| 13 | `enable-guild-join-renewal` | BOOL | yes | — | — | `no` | no |  |
| 14 | `enable-job` | STR | yes | — | — | `;` | no | new in 381 |

## 4. Semantics

| Key | Behaviour | Status |
| --- | --- | --- |
| `worker-thread` | `0` → 2 × CPU count I/O workers (`Network::Initialize`, `v3 = 2 * SysInfo::_ProcessorCount`). Rust: maps to the tokio worker count; same rule. | ✅ |
| `max-client` | Passed as the session limit to the client listener (`Network::OpenPort(1, client-port, …, maxClientSession)`). | ✅ use; ❓ whether an explicit `0` becomes 200000 or "no limit" |
| `client-session-hash-count` | Bucket count for `GateKeyManager` and `ClientManager` hash maps. Internal only: Rust may ignore it. | ✅ |
| `init-client-session-count` | Pre-allocated session pool size. Internal only: Rust may ignore it. | ✅ |
| `timer-interval` / `timer-max-time` | Returned by `LobbyGate::GetTimerInterval` / `GetTimerMaxTime`; drive the daemon timer queue (ms / s). `client-close-timeout` uses this queue in milliseconds. | ✅ close job: `AddJobTimer` `0x140013490` → `TimerQueue::Add` `0x14001F080`; reconnect use `0x140018FB0`. ❓ exact wheel phase |
| `dump-type` | Windows minidump flags. Rust: accept and ignore (log once). | ✅ |
| `max-idle-time-enable`, `disconnect-after-idle-time`, `max-idle-time` | Startup reads enable at config +0x54, disconnect at +0x55, and the low 16 bits of seconds at +0x58. It multiplies seconds by 1000. A watcher waits 5000 ms before each scan, including the first. Accept and receive set state 1 and the receive time. A scan moves 1→2 without an expiry check. State 2 moves to 3 only when `last_receive + timeout < now`. The next scan checks the same inequality in state 3, clears the state, and logs or force-closes. Receive restarts state 1. This is **not two full timeout periods**. | ✅ 381 config reader/setup `0x140013AB0`; scan `0x140036950`; first wait/cadence `0x140036B90`; accept `0x140018780`; receive `0x1400191A0` |
| `use-channel-delayed-release` | Session↔channel binding is released after a delay (comment: 10 s) instead of at once (`DelayedRelease`). | ✅ flag read; ❓ delay value |
| `alert-send-address` | `ip:port` passed to `AlertSender::init`. Empty → alerts off. | ✅ use; ❓ wire format of alerts (spec 0x later) |
| `cooldown-complete-delay` | Seconds to wait for all users to drop during a cooldown (`LobbyGate::cooldown`). | ✅ |
| `predefined-world-id` | Used as a 16-bit world id in LobbyDaemon messages (`SLOWORD(_predefinedWorldId)`). | ✅ |
| `cert` | NCSoft licence file read by `BloodCert`. **Not implemented in Rust** (agreed). Rust accepts the key and ignores it. | ✅ decision |
| `root-path`, `log-path`, `config-path` | Windows paths in shipped files. Rust accepts any path; logs go to `log-path`. | ✅ |
| Required key missing | ❓ Whether the daemon refuses to start or uses the table default (e.g. `predefined-world-id` has default `4001` and is also required). Needs `DaemonConfig::Load` / `XmlReader_sv64.dll`. | ❓ |
| Value outside min/max (rule tables) | `BnsCommonConfig::CheckMinMaxBoundary` / `ContentsRuleConfig::CheckMinMaxBoundary` reject it. ❓ fail or clamp. | ❓ |
| Reload | `LobbyGate::reload` re-reads files; only keys with `reload = yes` change at run time (here: `np-app-id`, `np-client-app-id`). | ✅ flags; ❓ trigger (pipe console command `CMD_RELOAD_REQ`) |

### Rust timer policy

- The close-job queue checks exact monotonic deadlines on each configured `timer-interval` tick. The first tick is one interval after `serve` starts. Missed ticks use `Skip`.
- Rust rejects `timer-interval = 0`. `timer-max-time` is a wheel storage horizon in the original. Rust uses a heap, so this value does not limit job duration.
- Original cadence source: `Daemon::timerMain` `0x140020AA0`. It waits for the first interval, then advances the queue once per tick. `TimerQueue::Add` `0x14001F080` divides the delay by the interval. `TimeExpired` `0x140043B80` advances the slot before draining it.
- ❓ TODO(spec 01 §4): exact original wheel phase and zero-slot wrap parity. Rust does not emulate a job that wraps a wheel slot when its delay is shorter than one interval.
- Idle scans use a separate 5000 ms cadence from server startup. A late accept waits for the next global scan. It does not run a missed scan at accept time. They also cover incomplete handshakes and partial frames. Disabled means no scans. Log-only reports once until another receive. Source: global watcher `0x140036950`, wait loop `0x140036B90`, accept state `0x140018780`.
- TODO(spec 01 §4): delayed release, alerts, cooldown, and runtime GUID reload remain unimplemented. Existing source evidence appears in the §4 table; exact implementation details remain ❓.

## 5. Rust mapping (proposal)

- One `serde` struct per file, field names = key names (`#[serde(rename = "...")]`), defaults from the tables above.
- Parse with `quick-xml`; collect `<option name value>` pairs into a map first, then fill the struct, so unknown keys are ignored and order does not matter.
- Validate min/max for the rule tables; behaviour on failure follows the open item in §4.
- CLI: `lobbygate --config-dir <dir>`; default = directory of the binary (matches `CONFIG_DIR_OPTION` once confirmed).

## 6. Rust step-1 policy (not original-binary evidence)

The step-1 loader implements these explicit safety choices. These choices do not
resolve the original-reader questions in §7.

| Item | Rust policy | Original behaviour |
| --- | --- | --- |
| Required keys | Apply a listed default. Reject a missing required key that has no default. Accept `cert` as text; never open it. | ❓ |
| BOOL | Accept only `yes` and `no`, with exact case. | Other spellings are ❓ |
| Key names | Match exact case. Do not trim names or values. Ignore unknown names. | Case and trimming are ❓ |
| Rule limits | Reject out-of-range values. Do not clamp. | Reject versus clamp is ❓ |
| Duplicate options | Reject them. | ❓ |
| XML input | Limit each file to 1 MiB. Reject DTDs and unexpected elements or attributes. | Original size and extension rules are ❓ |
| Worker count | Accept 0–1024. Zero selects twice the CPU count. | Zero rule is verified; upper limit is a Rust safety choice. |
| Client limit | Require a positive `max-client`. | Meaning of zero is ❓ |
| Numeric safety | Reject negative durations and counts. Ports must fit `u16`. | Original handling is ❓ |
| GUIDs | Accept canonical hexadecimal GUID text. Missing GUIDs stay unset. | Built-in GUIDs are not used in Rust. |
| Paths | Load all three files from `--config-dir`; default to the binary directory. Append tracing logs to `log-path/lobbygate.log`. | Original CLI spelling and search path are ❓ |

## 7. Open items

1. `XmlReader_sv64.dll`: BOOL spellings, case, whitespace, required-key handling.
2. Command-line options (`HELP_OPTION`, `CONFIG_DIR_OPTION`, `WINDOW_CONSOLE_OPTION`, …): exact spellings.
3. `max-client = 0` meaning.
4. Idle watcher units and first scan are resolved in §4. Rust uses a shared startup scan phase and skips missed scans. OS thread scheduling parity remains ❓.
5. Delayed-release delay value.
6. Where `ranking-service-mode` goes ❓. Resolved for the contents-rule values (incl. `enable-job`): they go to the client as XML lines in C107 (05 §5.3).
