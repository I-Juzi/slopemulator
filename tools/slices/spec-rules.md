# Spec rules (Opus IDA slice session)

You reverse one slice in IDA and write its spec. Your card is the user message.

- Open only the `.i64` copy that the card names. Never open another copy.
- Every behaviour claim cites an address or function. Mark claims that you did not verify with ❓.
- Do not copy keys, passwords, certificates or Hex-Rays code. Describe behaviour.
- Change only the paths that the card lists under "Files".
- Never read or probe the IDA licence. If IDA fails to open, write Blocked and stop.
- Do not commit. The gate commits.
- Write all text in ASD-STE100.

Context budget (each tool result is read again on every later turn):
- Each turn reads the full context again. The number of turns is the main cost. Target: 40 IDA calls or fewer for the whole slice.
- Work in batches. For each topic, first run 1 survey script that covers the whole topic: all handlers, their callees, xrefs, strings, constants and enums. Then run only the follow-up calls that the survey shows you need.
- Send independent tool calls in the same turn.
- Keep each IDA result under 3,000 tokens (about 7,000 characters). A batch result can use the full limit.
- Do not print full pseudocode. Filter it in Python: print only the lines with calls, strings, constants or the field that you need.
- Write the chapter in large edits (one topic per edit), not one fact per edit.
- Do not read files outside the worktree and the `.i64` copy.
- For Rust interfaces, run `python3 tools/rust_api.py <crate dir>`. Do not read Rust source files.
- Write facts into the spec file as soon as you verify them. Do not keep them only in the conversation.

## Original bugs (quirks)
- The original code has bugs, for example out-of-bounds reads, reads of uninitialized memory, wrong indexes and integer overflow. Game data and scripts can depend on what these bugs return. A port that fixes them silently breaks that content.
- Do not fix a bug in the spec. Write a "Quirk" row: address, what the code does, the value it actually gets (for example the next field in the struct, the old buffer contents, or zero), and which data or script can reach it.
- Give the quirk a short name (`quirk_<name>`), so that the Rust code and the tests can cite it.

## Exploits (do not copy)
- An exploit is a bug or a trust gap that a player can use for gain: item dupes, skipped cooldowns, wrong reward counts, actions without the right state or distance. The 2020 server comes from the retail era, so the retail exploits are probably in it.
- Do not copy an exploit. Write an "Exploit" row: address, what the original does, how a player can use it, and the fix. The server never trusts a value that comes from the client or from the client data files. It uses its own data and its own state.
- Give it a name (`exploit_<name>`). Check each item in `Docs/Plan/known-exploits.md` that your topic touches.
- If you are not sure if a bug is a quirk or an exploit, mark it ❓ and write both readings.

Last steps:
1. Unless the card says "No Rust card", write the Rust card for the next slice with `Docs/Plan/card-template.md`. Path: `Docs/Plan/cards/<slice id>-rust.md`. Give exact spec line ranges, Rust signatures and test cases.
2. Write `Docs/Plan/handoffs/<slice id>.md`, 30 lines or fewer: Done, Not done, ❓, Blocked.

## Order of work
- Write each verified fact into the spec at once. Do not keep facts only in the conversation.
- Write the Rust card early, before the last topic. Update it at the end. A slice without the card cannot chain to Rust.
- You have no general Bash. Do not try `tools/slice_runner.py`. Check the gate rules by hand.
