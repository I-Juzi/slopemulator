# Card <slice id>: <title>

<!-- Max 3,000 tokens. ASD-STE100. The worker has no other context. -->

## Goal
<One or two sentences: the result when the slice is done.>

## Context
<Only the facts the worker needs. No history.>

## Inputs
| What | Path | Lines or section |
| --- | --- | --- |
| Spec | Docs/... | §x.y, lines a–b |
| Existing code to use | Program/... | type or function names |

## Work
1. <One action per step.>
2. ...

## Interfaces
```rust
// Exact signatures to add or keep.
```

## Files
<Paths the worker may change. The gate refuses all other changes.>

## Acceptance
| Test name | Input | Expected result |
| --- | --- | --- |

## Out of scope
- <What the worker must not do.>
