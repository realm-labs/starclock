# Goal 06 Launch Prompt

Use this prompt to start or resume the persistent Goal 06 execution loop.

```text
Start or resume Goal 06: Combat Identity and Dynamic Per-Battle Assembly.

Continue until every terminal checklist item in
docs/goals/06-combat-identity-and-dynamic-assembly.md is complete. Do not stop
after planning, the digest migration, one dynamic-assembly fixture or a
passing unit test.

Read and obey, in order:
1. docs/goals/06-combat-identity-and-dynamic-assembly.md
2. docs/goals/06-combat-identity-and-dynamic-assembly-status.md
3. docs/goals/05-standard-universe-end-to-end-status.md
4. docs/28-standard-universe-integration-coverage.md
5. docs/27-standard-universe-end-to-end-integration.md
6. docs/26-mode-extension-and-evolution.md
7. docs/20-core-implementation-design.md
8. docs/19-activity-core-and-mode-extension.md
9. docs/16-replay-cli-and-engine-integration.md
10. docs/11-rule-ir-and-native-handlers.md
11. docs/10-lifecycle-and-resolution.md
12. docs/09-determinism-and-numerics.md
13. docs/08-engineering-standards.md
14. docs/07-configuration-pipeline.md
15. docs/06-rust-architecture.md

Execution loop:
- inspect the worktree, active persistent goal and Goal 06 ledger;
- verify the Goal 05 immutable snapshot before the first implementation batch;
- select the earliest unblocked Pending batch and mark only it InProgress;
- implement the complete batch with focused tests, docs and owned evidence;
- run format, focused Clippy/tests and the quick repository gate;
- run the full repository gate only at phase/release checkpoints;
- update the ledger with exact commands, hashes, timing and observed result;
- commit atomically with the required G06 batch ID;
- continue immediately to the next batch;
- mark the persistent goal complete only after G06-P4-B3 is committed, the
  immutable snapshot is registered and clean-worktree release verification
  passes.

Mandatory constraints:
- starclock-combat computes the combat-visible digest and never depends on
  Activity, build, data, replay, AI or mode crates;
- AssemblyDigest is opaque provenance, not a replacement for the computed
  combat input digest;
- a Battle consumes one immutable snapshot and never reads live Activity state;
- catalog composition is not repeated for every battle;
- cache state is non-authoritative, bounded and excluded from canonical hashes;
- invalid assembly and failed execution preserve Activity bytes, RNG counters
  and replay records;
- production CLI, Agent, MCP and replay reconstruction use one assembler;
- replay v2 remains historical and no v2 evidence is rewritten;
- do not implement the remaining Standard Universe content rules in this goal
  beyond the representative dynamic-assembly acceptance fixtures;
- use Python openpyxl plus Sora 0.3.0 for any workbook change;
- keep handwritten Rust files below 1,200 physical lines, split touched
  near-limit modules by responsibility and avoid convenience pub use;
- preserve unrelated user changes and keep focused acceptance within 1–3
  minutes through cache reuse and scoped checks.

If a batch exposes an architectural gap, update the Goal 06 plan and ledger in
that same batch. Do not weaken identity coverage, hide an assembly failure
behind a retained approximation, or treat a changed Activity inventory as
integrated unless the following real BattleSpec and battle hash prove it.
```
