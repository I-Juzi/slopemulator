# ids-2-rust handoff

## Done
- `packet-ids` is a dependency of GameDaemon (`Cargo.toml`, `Cargo.lock`).
- `build.rs` writes one `pub const <Name>: u16` for each `GameCmd` variant. `src/ids.rs` includes it as `ids::<Name>`. The value is `GameCmd::<Name>.id()`. A wrong name does not compile.
- Reason: the card says `packet-ids` has `pub const` modules. It has only enums, and an enum variant cannot be a `u16` `match` pattern.
- All dispatch `match` arms and all `*_COMMANDS` lists use `ids::` names: mod.rs, npc_trade, challenge, social, quest, quest2, mentoring, post, market, shop, misc, duel_ranking, season, arena_event, arena_lobby, chat (with `QQ_COMMANDS`), trade, items, items2-items5, skills, skills2, party, guild.
- Request and reply IDs in the call arguments (`pc_task`, `queue`, `board_gacha_*`, `handle_refiner`, `handle_item_pouch`, `handle_two_group`) use names in the dispatch arms.
- movement.rs: all reply, fail and queued IDs use names. skills.rs: all `command:` fields use names.
- Test `dispatch_ids_named` (command and output):
  `rg -n "^\s+[0-9]+( \| [0-9]+)* =>" Program/Server/GameDaemon/src/handlers`
  gives only value maps, not dispatch arms (for example `0 => ...`, `1 | 2 => ...`, token.rs:80 reason codes, session.rs:160 value table). No command arm is left.
- `cargo fmt`, `cargo test -p gamedaemon` (all pass, no test changed), `cargo clippy --all-targets -D warnings` pass.

## Not done
- Reply and fail IDs in `Outcome::reason(<n>, ..)` / `Outcome::reply(<n>, ..)` calls are still bare in skills.rs, skills2.rs, items*.rs, trade.rs, party*.rs, guild.rs, session.rs and other files that I did not touch in this stage. Find them with `rg -n "Outcome::(reason|reply)\([0-9]+" Program/Server/GameDaemon/src`.
- `StateChange::Queued(<n>)` and `ctx.queue(<n>, ..)` calls in the same files.
- The tests in `Program/Server/GameDaemon/tests/` still use bare IDs (they pass unchanged).
- Behaviour did not change. A later pass can use the same `ids::` names.

## ❓
- No number and name mismatch was found, so no `// packet-ids says <n>` note was needed.
- Does the gate want reply and fail IDs named too? The card says "all command IDs". They are `GameCmd` IDs, but there are about 1500 sites.
- Is a `build.rs` in GameDaemon acceptable? It reads `../PacketIds/src/generated/commands.rs`. An alternative is a `pub const` module in `packet-ids` (outside this card).

## Blocked
- None. No abuse case found (see no file in `Docs/Plan/abuse-notes/` for this slice).
