# Goal 05 Launch Prompt

Use this prompt to start or resume the persistent Goal 05 execution loop.

```text
Start or resume Goal 05: Standard Universe End-to-End Integration.

Continue until every terminal checklist item in
docs/goals/05-standard-universe-end-to-end.md is complete. Do not stop after
planning, scaffolding, one vertical slice or a passing unit test.

Read and obey, in order:
1. docs/goals/05-standard-universe-end-to-end.md
2. docs/goals/05-standard-universe-end-to-end-status.md
3. docs/27-standard-universe-end-to-end-integration.md
4. docs/26-mode-extension-and-evolution.md
5. docs/25-standard-universe-runtime-design.md
6. docs/19-activity-core-and-mode-extension.md
7. docs/16-replay-cli-and-engine-integration.md
8. docs/12-modifiers-and-snapshots.md
9. docs/11-rule-ir-and-native-handlers.md
10. docs/10-lifecycle-and-resolution.md
11. docs/09-determinism-and-numerics.md
12. docs/08-engineering-standards.md
13. docs/07-configuration-pipeline.md
14. docs/06-rust-architecture.md
15. docs/goals/04-standard-universe-runtime-status.md

Execution loop:
- inspect the worktree, active persistent goal and Goal 05 ledger;
- select the earliest unblocked Pending batch and mark only it InProgress;
- implement the complete batch, including tests, docs and generated policy or
  evidence owned by that batch;
- run focused format/lint/tests and the quick repository gate;
- run the full repository gate only at phase/release checkpoints;
- update the ledger with concrete commands, hashes and results;
- commit atomically with the required G05 batch ID;
- continue immediately to the next batch;
- mark the persistent goal complete only after G05-P5-B3 is committed and a
  clean-worktree release verification passes.

Mandatory constraints:
- never edit or regenerate frozen Goal 01–04 release evidence to make Goal 05
  pass;
- production CLI, agent and MCP completion may not use
  verified-reference-projection-v1 or fabricated Won results;
- mode handlers return ordinary validated Activity/combat operations and never
  mutate aggregate state directly;
- starclock-combat cannot depend on starclock-mode-universe;
- costs, effects, interaction consumption and graph transition are atomic;
- all RNG, fixed-point, canonical hashing and replay revision rules remain
  deterministic and explicit;
- retain Version 4.4 approximation markers instead of inventing exact values;
- use Python openpyxl plus Sora 0.3.0 for workbook changes;
- keep handwritten Rust files below 1,200 physical lines, split by
  responsibility and avoid convenience pub use;
- preserve unrelated user changes;
- daily focused acceptance must reuse caches and stay within 1–3 minutes.

When a batch exposes an architectural gap, update the Goal 05 design and ledger
in that same batch. Do not weaken acceptance, convert an integration test into
an evaluator-only test, or describe an effect plan as applied behavior.
```
