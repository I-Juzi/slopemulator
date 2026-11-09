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
- Keep each IDA result under 1,500 tokens (about 3,400 characters).
- Do not print full pseudocode. Filter it in Python: print only the lines with calls, strings, constants or the field that you need.
- Put many functions in one `execute_python` call. Each call must answer one question.
- Do not read files outside the worktree and the `.i64` copy.
- For Rust interfaces, run `python3 tools/rust_api.py <crate dir>`. Do not read Rust source files.
- Write facts into the spec file as soon as you verify them. Do not keep them only in the conversation.

Last steps:
1. Unless the card says "No Rust card", write the Rust card for the next slice with `Docs/Plan/card-template.md`. Path: `Docs/Plan/cards/<slice id>-rust.md`. Give exact spec line ranges, Rust signatures and test cases.
2. Write `Docs/Plan/handoffs/<slice id>.md`, 30 lines or fewer: Done, Not done, ❓, Blocked.

## Order of work
- Write each verified fact into the spec at once. Do not keep facts only in the conversation.
- Write the Rust card early, before the last topic. Update it at the end. A slice without the card cannot chain to Rust.
- You have no general Bash. Do not try `tools/slice_runner.py`. Check the gate rules by hand.
