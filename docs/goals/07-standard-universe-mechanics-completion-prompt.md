# Goal 07 Launch Prompt

Use this prompt only after Goal 06 has completed and its immutable release
snapshot is registered.

```text
Start or resume Goal 07: Complete Standard Universe Mechanics.

First verify that docs/goals/06-combat-identity-and-dynamic-assembly-status.md
is Complete and that policy/release-snapshots.json contains the Goal 06
completion commit/tree. If either condition fails, do not start Goal 07 and do
not duplicate Goal 06 work.

Once unblocked, continue until every terminal checklist item in
docs/goals/07-standard-universe-mechanics-completion.md is complete. Do not
stop after partition generation, one Path, one Curio family, a seeded run or a
passing evaluator test.

Read and obey, in order:
1. docs/goals/07-standard-universe-mechanics-completion.md
2. docs/goals/07-standard-universe-mechanics-completion-status.md
3. docs/goals/06-combat-identity-and-dynamic-assembly-status.md
4. docs/28-standard-universe-integration-coverage.md
5. docs/27-standard-universe-end-to-end-integration.md
6. docs/25-standard-universe-runtime-design.md
7. docs/23-standard-simulated-universe-reference.md
8. docs/12-modifiers-and-snapshots.md
9. docs/11-rule-ir-and-native-handlers.md
10. docs/10-lifecycle-and-resolution.md
11. docs/05-effects-and-resources.md
12. docs/04-toughness-and-break.md
13. docs/03-damage-and-sustain.md
14. docs/02-action-order-and-turns.md
15. docs/09-determinism-and-numerics.md
16. docs/08-engineering-standards.md
17. docs/07-configuration-pipeline.md
18. docs/26-mode-extension-and-evolution.md
19. docs/sources.md

Execution loop:
- inspect the worktree, active persistent goal, Goal 07 ledger and generated
  content partition manifest;
- select the earliest unblocked fixed batch or generated Snn sub-batch and
  mark only it InProgress;
- implement the complete partition: Excel, Sora output, lowering, runtime,
  focused real-state/event/hash tests, provenance and coverage;
- run format, focused Clippy/tests and the quick repository gate;
- run the full repository gate only at phase/release checkpoints;
- update the fixed ledger, expanded sub-batch ledger and exact-once coverage;
- commit atomically using the exact G07 batch/sub-batch ID;
- continue immediately to the next ordered partition;
- mark the persistent goal complete only after every generated sub-batch and
  G07-P7-B3 are committed, zero missing-runtime approximations remain, the
  immutable snapshot is registered and clean release verification passes.

Mandatory constraints:
- use the Goal 05 2,201/786/78 assignment as the starting oracle and never
  reduce the denominator to make coverage pass;
- use Goal 06 per-battle dynamic assembly; do not add a second assembly path;
- numeric values may be exact or an approved evidenced approximation;
  mechanic behavior may not be replaced by a generic proxy;
- insufficient public evidence for mechanic behavior blocks the partition and
  must be recorded/escalated instead of guessed;
- a typed evaluator, workbook row, route completion or effect-plan object does
  not count as executable behavior;
- all combat mechanics emit generic Rule IR/native-handler operations through
  the normal Battle resolver; never branch on Universe/content IDs in shared
  resolver code;
- every noncombat cost, effect, inventory mutation and transition commits
  atomically; external minigames use explicit result commands;
- edit production Excel with Python openpyxl and export through pinned Sora
  0.3.0; JSON remains staging/debug only;
- record provenance, confidence and approximation policy for factual changes;
- preserve fixed-point, deterministic RNG, canonical ordering, replay and
  rollback guarantees;
- keep handwritten Rust files below 1,200 physical lines, split by
  responsibility and avoid convenience pub use;
- preserve unrelated user changes;
- keep each generated partition within its rule/choice/enemy/native-handler
  cap and keep daily focused verification within 1–3 minutes.

When a content partition exposes a missing shared primitive, implement and
test that primitive in the same partition only if it is narrow and already
covered by the Phase 1 capability contract. Otherwise record the gap, add a
bounded prerequisite batch before the affected partition and update the
ledger. Do not hide the gap with RetainedApproximation.
```
