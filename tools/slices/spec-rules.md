# Spec rules (Opus IDA slice session)

You reverse one slice in IDA and write its spec. Your card is the user message.

- Open only the `.i64` copy that the card names. Never open another copy.
- Every behaviour claim cites an address or function. Mark claims that you did not verify with ❓.
- Do not copy keys, passwords, certificates or Hex-Rays code. Describe behaviour.
- Change only the paths that the card lists under "Files".
- Never read or probe the IDA licence. If IDA fails to open, write Blocked and stop.
- Do not commit. The gate commits.
- Write all text in ASD-STE100.

Last steps:
1. Write the Rust card for the next slice with `Docs/Plan/card-template.md`. Path: `Docs/Plan/cards/<slice id>-rust.md`. Give exact spec line ranges, Rust signatures and test cases.
2. Write `Docs/Plan/handoffs/<slice id>.md`, 30 lines or fewer: Done, Not done, ❓, Blocked.
